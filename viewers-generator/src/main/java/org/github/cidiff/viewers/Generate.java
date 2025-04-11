package org.github.cidiff.viewers;

import com.github.mustachejava.DefaultMustacheFactory;
import com.github.mustachejava.Mustache;
import org.github.cidiff.Action;
import org.github.cidiff.DiffClient;
import org.github.cidiff.Line;
import org.github.cidiff.LogDiffer;
import org.github.cidiff.LogParser;
import org.github.cidiff.Metric;
import org.github.cidiff.Options;
import org.github.cidiff.Pair;

import java.io.BufferedWriter;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.FileVisitOption;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;

public class Generate {

	public static final String SUCCESS_FILE = "success.log";
	public static final String FAILURE_FILE = "failure.log";

	public static void main(String[] args) {
		if (args.length != 1) {
			System.out.println("Usage: java -jar exp.jar <dataset_path>");
			return;
		}
		final String dataset = args[0];

		List<Path> directories = collectDirectories(dataset);
		LogParser parser = LogParser.Type.GITHUB.construct();
		LogDiffer differSeed = LogDiffer.Algorithm.SEED.construct();
		LogDiffer differLcs = LogDiffer.Algorithm.LCS.construct();

		Options optionsSeed = new Options();
		Options optionsLcs = new Options().with(Options.METRIC, Metric.EQUALITY);

		File fileSeed = new File("../viewers/output/alpha/");
		fileSeed.mkdirs();
		File fileLcs = new File("../viewers/output/beta/");
		fileLcs.mkdirs();
		System.out.printf("directories found: %d%n", directories.size());
		long seed = 123456L;
		Random random = new Random(seed);
		for (int i = 0; i < 100; i++) {
			int n = random.nextInt(directories.size());
			Path dir = directories.get(n);
			List<Line> leftLines = parser.parse(dir.resolve(SUCCESS_FILE).toString(), optionsSeed);
			List<Line> rightLines = parser.parse(dir.resolve(FAILURE_FILE).toString(), optionsSeed);
			if (!leftLines.isEmpty() && !rightLines.isEmpty() && leftLines.size() < 30_000 && rightLines.size() < 30_000) {
				System.out.printf("%d %s", i, dir);
				System.out.print(" seed");
				diffIt(differSeed, leftLines, rightLines, optionsSeed.with(Options.OUTPUT_PATH, "../viewers/output/alpha/diff" + i + ".html"));
				System.out.print(" lcs");
				diffIt(differLcs, leftLines, rightLines, optionsLcs.with(Options.OUTPUT_PATH, "../viewers/output/beta/diff" + i + ".html"));
				System.out.println();
			} else {
				--i;
			}
		}

		DefaultMustacheFactory factory = new DefaultMustacheFactory();
		Mustache mustache = factory.compile("evaluation.mustache");
		File f = new File("../viewers/output/diff/");
		f.mkdirs();
		for (int i = 0; i < 100; i++) {
			try {
				BufferedWriter writer = new BufferedWriter(new FileWriter("../viewers/output/diff/viewer" + i + ".html"));
				mustache.execute(writer, Map.of("num", i, "next", i + 1, "prev", i - 1));
				writer.close();
			} catch (IOException e) {
				throw new RuntimeException(e);
			}
		}

	}

	public static void diffIt(LogDiffer differ, List<Line> left, List<Line> right, Options options) {
		Pair<List<Action>> actionsSeed = differ.diff(left, right, options);
		var lines = new Pair<>(left, right);
		DiffClient client = DiffClient.Type.FILTERED.construct(lines, actionsSeed);
		client.execute(options);
	}

	public static List<Path> collectDirectories(String dataset) {
		List<Path> directories = new ArrayList<>();

		SimpleFileVisitor<Path> walker = new SimpleFileVisitor<>() {

			@Override
			public FileVisitResult postVisitDirectory(Path dir, IOException exc) throws IOException {
				Path success = dir.resolve(SUCCESS_FILE);
				Path failure = dir.resolve(FAILURE_FILE);
				if (failure.toFile().exists() && success.toFile().exists()) {
					directories.add(dir);
				}
				return super.postVisitDirectory(dir, exc);
			}
		};

		try {
			Files.walkFileTree(Path.of(dataset), Set.of(FileVisitOption.FOLLOW_LINKS), 10, walker);
		} catch (IOException e) {
			throw new RuntimeException(e);
		}
		return directories;
	}

}
