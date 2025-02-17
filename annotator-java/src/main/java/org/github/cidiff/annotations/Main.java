package org.github.cidiff.annotations;

import org.github.cidiff.Action;
import org.github.cidiff.Line;
import org.github.cidiff.LogDiffer;
import org.github.cidiff.LogParser;
import org.github.cidiff.Metric;
import org.github.cidiff.Options;
import org.github.cidiff.Pair;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.IntStream;
import java.util.stream.Stream;

public class Main {

	public static void main(String[] args) throws IOException {
		if (args.length != 2) {
			System.out.println("Usage: java -jar exp.jar <dataset_path> <logs_path>");
			return;
		}
		final String DATASET = args[0];
		final String PATHS = args[1];

		List<String> paths = new BufferedReader(new FileReader(PATHS)).lines().toList();
		LogParser parser = LogParser.Type.GITHUB.construct();
		LogDiffer differSeed = LogDiffer.Algorithm.SEED.construct();
		LogDiffer differLCS = LogDiffer.Algorithm.LCS.construct();

		Options optSeed = new Options();
		Options optLCS = new Options().with(Options.METRIC, Metric.EQUALITY);
		Options optDrain = new Options().with(Options.METRIC, Metric.DRAINSIM);

		BufferedWriter writer = new BufferedWriter(new FileWriter("selection.csv"));
		writer.write("path,type,line\n");
		for (String path : paths) {
			// create a new drain parser every loop to reset its internal data (parsed lines tree)
			System.out.print(path + " ");
			List<Line> leftLines = parser.parse(DATASET + "/" + path + "/success.log", optSeed);
			List<Line> rightLines = parser.parse(DATASET + "/" + path + "/failure.log", optSeed);
			diff(leftLines, rightLines, optSeed, differSeed, path, "cidiff", writer);
			diff(leftLines, rightLines, optLCS, differLCS, path, "lcs-diff", writer);
			// bigram with raw lines
			bigramDiff(leftLines, rightLines, path, "bigram", writer);
			// bigram with lines parsed by drain
			LogParser parserDrain = LogParser.Type.DRAIN.construct();
			// ignore the first parse on the left lines, because we don't have the templates correctly setup yet
			parserDrain.parse(DATASET + "/" + path + "/success.log", optSeed);
			List<Line> parsedRightLines = parserDrain.parse(DATASET + "/" + path + "/failure.log", optSeed);
			// To replace the lines by their template, the parser must have the two logs to be able to find presumably correct templates.
			// However, the first #parse() has only the first log. Now that both logs are inside it's internal tree, parse another time the first log.
			// This won't change anything about the templates (supposedly).
			List<Line> parsedLeftLines = parserDrain.parse(DATASET + "/" + path + "/success.log", optSeed);
			bigramDiff(parsedLeftLines, parsedRightLines, path, "bigram-drain", writer);
			// at that point, the drain parser should have the lines correctly parsed, so we should be able to use Drain#INSTANCE now
			diff(leftLines, rightLines, optDrain, differSeed, path, "cidiff-drainsim", writer);
			System.out.println();
		}
		writer.close();
		System.out.println();
		System.out.println("======PARAMETER SENSIBILITY======");
		parameterSensibility(DATASET, paths);
	}

	private static void bigramDiff(List<Line> leftLines, List<Line> rightLines, String path, String type, BufferedWriter writer) {
		System.out.print(type + " ");
		Set<Pair<String>> bigramsL = new HashSet<>();
		Set<Pair<String>> bigramsR = new HashSet<>();
		for (int i = 0; i < leftLines.size() - 1; i++) {
			bigramsL.add(new Pair<>(leftLines.get(i).value(), leftLines.get(i + 1).value()));
		}
		for (int i = 0; i < rightLines.size() - 1; i++) {
			bigramsR.add(new Pair<>(rightLines.get(i).value(), rightLines.get(i + 1).value()));
		}
		bigramsR.removeIf(bigramsL::contains);  // remove right bigrams present in the left bigrams
		List<String> bigrams = bigramsR.stream().flatMap(b -> Stream.of(b.left(), b.right())).toList();

		for (Line line : rightLines) {
			if (bigrams.contains(line.value())) {
				try {
					writer.write("%s,%s,%d%n".formatted(path, type, line.index()));
				} catch (IOException e) {
					throw new RuntimeException(e);
				}
			}
		}
	}

	public static void diff(List<Line> leftLines, List<Line> rightLines, Options options, LogDiffer differ, String path, String type, BufferedWriter writer) {
		System.out.print(type + " ");
		Pair<List<Action>> diff = differ.diff(leftLines, rightLines, options);
		List<Action> actions = diff.right();
		List<Integer> l = new ArrayList<>();
		for (int i = 0; i < actions.size(); i++) {
			if (actions.get(i).type() == Action.Type.ADDED) {
				l.add(i);
//				l.add(i-1);
//				l.add(i+1);
			}
		}
		l.stream().distinct().forEach(i -> {
			try {
				writer.write("%s,%s,%d%n".formatted(path, type, i));
			} catch (IOException ignored) {
			}
		});
	}

	public static void parameterSensibility(String dataset, List<String> paths) throws IOException {
		LogParser parser = LogParser.Type.GITHUB.construct();
		LogDiffer differ = LogDiffer.Algorithm.SEED.construct();

		Options opt = new Options();

		BufferedWriter writer = new BufferedWriter(new FileWriter("parameters.csv"));
		writer.write("path,type,line\n");
		int n = 0;
		for (String path : paths) {
			List<Line> leftLines = parser.parse(dataset + "/" + path + "/success.log", opt);
			List<Line> rightLines = parser.parse(dataset + "/" + path + "/failure.log", opt);
			for (double minSimilarity = 0.0; minSimilarity <= 1.05; minSimilarity += 0.1) {
				for (double minQGramSimilarity = 0.0; minQGramSimilarity <= 1.05; minQGramSimilarity += 0.1) {
					System.out.printf("\r %d/100 %s %.1f %.1f%n", n, path, minSimilarity, minQGramSimilarity);
					Options options = new Options().with(Options.REWRITE_MIN, minSimilarity).with(Options.QGRAM_MIN, minQGramSimilarity);
					Pair<List<Action>> diff = differ.diff(leftLines, rightLines, options);
					List<Action> actions = diff.right();
					double[] sims = new double[]{minSimilarity, minQGramSimilarity};
					IntStream.range(0, actions.size())
							.filter(i -> actions.get(i).type() == Action.Type.ADDED)
							.distinct()
							.forEach(i -> {
								try {
									writer.write("%s,l%.1f-q%.1f,%d%n".formatted(path, sims[0], sims[1], i));
								} catch (IOException ignored) {
								}
							});
				}
			}
			System.out.println();
			++n;
		}
		writer.close();
	}

}
