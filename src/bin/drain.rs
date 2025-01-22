use apollo::parse_file;
use clap::{command, Parser};
use lru::LruCache;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    fs::{self, File},
    io::{BufWriter, Write},
    num::NonZeroUsize,
    path::Path,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    dataset: String,
    log_paths: String,
    inclue_success: bool,
}
/// Compute the drain parser on a set of files
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dataset_path = Path::new(&args.dataset);
    let mut output = csv::WriterBuilder::new().from_path("drained.csv")?;
    //let mut o = csv::WriterBuilder::new().from_path("drain.csv")?;
    //o.write_record(["path", "type", "line"])?;
    output.write_record(["path", "lineno", "line", "cluster", "cluster_size"])?;
    for path in fs::read_to_string(Path::new(&args.log_paths))?.lines() {
        let failure_path = dataset_path.join(path).join("failure.log");
        let failure_lines = fs::read_to_string(Path::new(dataset_path).join(&failure_path))
            .map(parse_file)
            .unwrap_or_default();
        let mut drain = Drain::new(None, 4, 0.5, 100, "<*>".to_string())?;
        for line in failure_lines.iter() {
            drain.train(line);
        }
        //let mut drain2 = Drain::new(None, 4, 0.5, 100, "<*>".to_string())?;
        //for line in failure_lines.iter() {
        //    drain2.train(line);
        //}
        if args.inclue_success {
            let success_path = dataset_path.join(path).join("success.log");
            let success_lines = fs::read_to_string(Path::new(dataset_path).join(&success_path))
                .map(parse_file)
                .unwrap_or_default();
            //for line in success_lines.iter() {
            //    drain2.train(line);
            //}
            for line in success_lines {
                drain.train(line);
            }
        }
        let mut n = 0;
        for (i, line) in failure_lines.iter().enumerate() {
            let tokens = tokenize(line);
            if let Some(cluster) = drain.tree_search(&tokens, drain.sim_th, false) {
                if cluster.size > 1 {
                    output.write_record([
                        path,
                        &i.to_string(),
                        line,
                        &cluster.to_string(),
                        &cluster.size.to_string(),
                    ])?;
                    //o.write_record([path, "drain", &i.to_string()])?;
                } else {
                    n += 1;
                }
            } else {
                n += 1;
            }
        }

        println!("{} not found 1: {}/{}", path, n, failure_lines.len());
    }

    //o.flush()?;
    output.flush()?;
    Ok(())
}


// Code below this line is the Drain implementation from https://github.com/ynqa/logu

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LogCluster {
    log_template_tokens: Vec<String>,
    pub cluster_id: usize,
    pub size: usize,
}

impl Display for LogCluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.log_template_tokens.join(" "))
    }
}

#[derive(Clone, Default)]
pub struct Node {
    key_to_child_node: HashMap<String, Node>,
    cluster_ids: Vec<usize>,
}

pub struct Drain {
    id_to_cluster: LruCache<usize, LogCluster>,

    max_node_depth: usize,

    /// Similarity threshold.
    /// A new log cluster will be created
    /// if the similarity of tokens for log message is below this.
    sim_th: f32,

    /// Maximum number of children within a node.
    max_children: usize,

    cluster_counter: usize,

    root: Node,

    param_str: String,
}

impl Debug for Drain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id_to_cluster: HashMap<_, _> = self.id_to_cluster.iter().map(|(k, v)| (*k, v.clone())).collect();

        fn fmt_node(
            node: &Node,
            f: &mut std::fmt::Formatter<'_>,
            depth: usize,
            id_to_cluster: &HashMap<usize, LogCluster>,
        ) -> std::fmt::Result {
            for _ in 0..depth {
                write!(f, "  ")?;
            }
            writeln!(f, "Node {{ cluster_ids: {:?} }}", node.cluster_ids)?;
            for cluster_id in &node.cluster_ids {
                if let Some(cluster) = id_to_cluster.get(cluster_id) {
                    for _ in 0..depth + 1 {
                        write!(f, "  ")?;
                    }
                    writeln!(
                        f,
                        "id: {}, log_template_tokens: {:?}",
                        cluster.cluster_id, cluster.log_template_tokens
                    )?;
                }
            }
            for (key, child) in &node.key_to_child_node {
                for _ in 0..depth + 1 {
                    write!(f, "  ")?;
                }
                writeln!(f, "key: {}", key)?;
                fmt_node(child, f, depth + 1, id_to_cluster)?;
            }
            Ok(())
        }

        writeln!(f, "Drain {{")?;
        fmt_node(&self.root, f, 1, &id_to_cluster)?;
        writeln!(f, "}}")
    }
}

impl Default for Drain {
    fn default() -> Self {
        Self {
            id_to_cluster: LruCache::unbounded(),
            max_node_depth: 2,
            sim_th: 0.4,
            max_children: 100,
            cluster_counter: 0,
            root: Node::default(),
            param_str: "<*>".to_string(),
        }
    }
}

impl Drain {
    pub fn new(
        max_clusters: Option<usize>,
        max_node_depth: usize,
        sim_th: f32,
        max_children: usize,
        param_str: String,
    ) -> anyhow::Result<Self> {
        let id_to_cluster = match max_clusters {
            Some(max_clusters) => LruCache::new(NonZeroUsize::new(max_clusters).unwrap()),
            None => LruCache::unbounded(),
        };

        Ok(Self {
            id_to_cluster,
            max_node_depth,
            sim_th,
            max_children,
            cluster_counter: 0,
            root: Node::default(),
            param_str,
        })
    }

    pub fn clusters(&self) -> Vec<&LogCluster> {
        self.id_to_cluster.iter().map(|(_, v)| v).collect()
    }

    pub fn train<T: AsRef<str>>(&mut self, log_message: T) -> LogCluster {
        let tokens = tokenize(log_message.as_ref());
        match self.tree_search(&tokens, self.sim_th, false) {
            Some(mut match_cluster) => {
                match_cluster.log_template_tokens =
                    self.create_template(&tokens, &match_cluster.log_template_tokens);
                match_cluster.size += 1;
                self.id_to_cluster
                    .put(match_cluster.cluster_id, match_cluster.clone());
                match_cluster
            }
            None => {
                self.cluster_counter += 1;
                let mut match_cluster = LogCluster {
                    log_template_tokens: tokens,
                    cluster_id: self.cluster_counter,
                    size: 1,
                };
                self.id_to_cluster
                    .put(match_cluster.cluster_id, match_cluster.clone());
                self.add_seq_to_prefix_tree(&mut match_cluster);
                match_cluster
            }
        }
    }

    pub fn tree_search(
        &mut self,
        tokens: &[String],
        sim_th: f32,
        include_params: bool,
    ) -> Option<LogCluster> {
        let token_count = tokens.len();

        let mut cur_node = self.root.key_to_child_node.get(&token_count.to_string())?;
        if token_count == 0 {
            return self.id_to_cluster.get(&cur_node.cluster_ids[0]).cloned();
        }

        let mut cur_node_depth = 1;
        for token in tokens {
            // At max depth.
            if cur_node_depth == self.max_node_depth {
                break;
            }

            // At last token.
            if cur_node_depth == token_count {
                break;
            }

            cur_node = cur_node
                .key_to_child_node
                .get(token)
                .or_else(|| cur_node.key_to_child_node.get(&self.param_str))?;

            cur_node_depth += 1;
        }
        self.fast_match(&cur_node.cluster_ids.clone(), tokens, sim_th, include_params)
    }

    fn fast_match(
        &mut self,
        cluster_ids: &[usize],
        tokens: &[String],
        sim_th: f32,
        include_params: bool,
    ) -> Option<LogCluster> {
        let mut match_cluster = None;
        let mut max_cluster = None;

        let mut max_sim = -1.0;
        let mut max_param_count = -1;
        for id in cluster_ids {
            let cluster = self.id_to_cluster.get(id).cloned();
            if let Some(cluster) = cluster {
                let (cur_sim, param_count) =
                    self.get_seq_distance(tokens, &cluster.log_template_tokens, include_params);
                if cur_sim > max_sim || (cur_sim == max_sim && param_count > max_param_count) {
                    max_sim = cur_sim;
                    max_param_count = param_count;
                    max_cluster = Some(cluster);
                }
            }
        }
        if max_sim >= sim_th {
            match_cluster = max_cluster;
        }
        match_cluster
    }

    fn get_seq_distance(&self, seq1: &[String], seq2: &[String], include_params: bool) -> (f32, isize) {
        let mut sim_tokens = 0;
        let mut param_count = 0;

        for (token1, token2) in seq1.iter().zip(seq2.iter()) {
            if token1 == &self.param_str {
                param_count += 1;
            } else if token1 == token2 {
                sim_tokens += 1;
            }
        }
        if include_params {
            sim_tokens += param_count;
        }
        (sim_tokens as f32 / seq1.len() as f32, param_count)
    }

    fn add_seq_to_prefix_tree(&mut self, cluster: &mut LogCluster) {
        let token_count = cluster.log_template_tokens.len();
        let token_count_str = token_count.to_string();

        let mut cur_node: &mut Node = self.root.key_to_child_node.entry(token_count_str).or_default();

        if token_count == 0 {
            cur_node.cluster_ids.push(cluster.cluster_id);
            return;
        }

        let mut current_depth = 1;
        for token in cluster.log_template_tokens.iter() {
            if current_depth >= self.max_node_depth || current_depth >= token_count {
                let mut new_cluster_ids = Vec::new();
                for cluster_id in cur_node
                    .cluster_ids
                    .iter()
                    .filter(|cluster_id| self.id_to_cluster.contains(cluster_id))
                {
                    new_cluster_ids.push(*cluster_id);
                }
                new_cluster_ids.push(cluster.cluster_id);
                cur_node.cluster_ids = new_cluster_ids;
                break;
            }

            if !cur_node.key_to_child_node.contains_key(token) {
                if !has_number(token) {
                    if cur_node.key_to_child_node.contains_key(&self.param_str) {
                        if cur_node.key_to_child_node.len() < self.max_children {
                            let new_node = Node::default();
                            cur_node.key_to_child_node.insert(token.clone(), new_node);
                            cur_node = cur_node.key_to_child_node.get_mut(token).unwrap();
                        } else {
                            cur_node = cur_node.key_to_child_node.get_mut(&self.param_str).unwrap();
                        }
                    } else if cur_node.key_to_child_node.len() + 1 < self.max_children {
                        let new_node = Node::default();
                        cur_node.key_to_child_node.insert(token.clone(), new_node);
                        cur_node = cur_node.key_to_child_node.get_mut(token).unwrap();
                    } else if cur_node.key_to_child_node.len() + 1 == self.max_children {
                        let new_node = Node::default();
                        cur_node
                            .key_to_child_node
                            .insert(self.param_str.clone(), new_node);
                        cur_node = cur_node.key_to_child_node.get_mut(&self.param_str).unwrap();
                    } else {
                        cur_node = cur_node.key_to_child_node.get_mut(&self.param_str).unwrap();
                    }
                } else if !cur_node.key_to_child_node.contains_key(&self.param_str) {
                    let new_node = Node::default();
                    cur_node
                        .key_to_child_node
                        .insert(self.param_str.clone(), new_node);
                    cur_node = cur_node.key_to_child_node.get_mut(&self.param_str).unwrap();
                } else {
                    cur_node = cur_node.key_to_child_node.get_mut(&self.param_str).unwrap();
                }
            } else {
                cur_node = cur_node.key_to_child_node.get_mut(token).unwrap();
            }

            current_depth += 1;
        }
    }

    fn create_template(&self, seq1: &[String], seq2: &[String]) -> Vec<String> {
        let mut new_template_tokens = Vec::new();
        for (token1, token2) in seq1.iter().zip(seq2.iter()) {
            if token1 == token2 {
                new_template_tokens.push(token1);
            } else {
                new_template_tokens.push(&self.param_str);
            }
        }
        new_template_tokens.iter().map(|s| s.to_string()).collect()
    }
}

fn has_number(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

fn tokenize(log_message: &str) -> Vec<String> {
    log_message.split_whitespace().map(|s| s.to_string()).collect()
}
