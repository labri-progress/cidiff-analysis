package org.github.cidiff.benchmark;

import org.github.cidiff.Action;
import org.github.cidiff.Line;
import org.github.cidiff.LogDiffer;
import org.github.cidiff.LogParser;
import org.github.cidiff.Metric;
import org.github.cidiff.Options;
import org.github.cidiff.Pair;
import org.github.cidiff.Utils;

import java.io.BufferedWriter;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.FileVisitOption;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.time.Duration;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.function.Supplier;

public class Benchmark {

	private static final int LOOPS = 3;
	public static final String SUCCESS_FILE = "success.log";
	public static final String FAILURE_FILE = "failure.log";
	public static final int TIMEOUT = 600;

	public static BufferedWriter logger;

	private static Pair<List<Action>> callWithTimeout(Supplier<Pair<List<Action>>> func) {
		ExecutorService executor = Executors.newSingleThreadExecutor();
		Future<Pair<List<Action>>> future = executor.submit(func::get);
		try {
			Pair<List<Action>> r = future.get(TIMEOUT, TimeUnit.SECONDS);
			executor.shutdownNow();
			return r;
		} catch (InterruptedException | ExecutionException | TimeoutException ignored) {
			future.cancel(true);
			executor.shutdownNow();
			return new Pair<>(new ArrayList<>(), new ArrayList<>());
		} finally {
			executor.shutdownNow();
		}
	}

	public static void main(String[] args) throws IOException {
		if (args.length != 1) {
			System.out.println("Usage: java -jar exp.jar <dataset_path>");
			return;
		}
		final Path dataset = Path.of(args[0]);
		// with 16Go of ram, it took 96H54M53S 
		List<Path> directories = collectDirectories(dataset);

		File file = new File("../csv/benchmark.csv");
		File logFile = new File("log.txt");

		BufferedWriter writer = new BufferedWriter(new FileWriter(file));
		logger = new BufferedWriter(new FileWriter(logFile));

		writer.write("directory,type,duration,lines-left,lines-right,actions,added,deleted,updated,moved-unchanged,moved-updated,similar-groups,similar-groups-left,similar-groups-right,runs\n");

		LogParser parser = LogParser.Type.GITHUB.construct();
		LogDiffer seed = LogDiffer.Algorithm.SEED.construct();
		LogDiffer lcs = LogDiffer.Algorithm.LCS.construct();

		Options optionsSeed = new Options();
		Options optionsLcs = new Options().with(Options.METRIC, Metric.EQUALITY);
		LocalDateTime start = LocalDateTime.now();

		logger.write("started at " + DateTimeFormatter.ISO_LOCAL_DATE_TIME.format(start));
		logger.newLine();

		int size = directories.size();
		for (int i = 0; i < size; i++) {
			Path dir = directories.get(i);
			List<Line> leftLines = parser.parse(dir.resolve(SUCCESS_FILE).toString(), optionsSeed);
			List<Line> rightLines = parser.parse(dir.resolve(FAILURE_FILE).toString(), optionsSeed);
			if (!leftLines.isEmpty() && !rightLines.isEmpty()) {
				if (i % 2 == 0) {
					compute(dataset, i, size, "seed", dir, seed, leftLines, rightLines, optionsSeed, writer, logger);
					compute(dataset, i, size, "lcs", dir, lcs, leftLines, rightLines, optionsLcs, writer, logger);
				} else {
					compute(dataset, i, size, "lcs", dir, lcs, leftLines, rightLines, optionsLcs, writer, logger);
					compute(dataset, i, size, "seed", dir, seed, leftLines, rightLines, optionsSeed, writer, logger);
				}
			}
			System.gc();
		}

		LocalDateTime end = LocalDateTime.now();

		writer.close();
		logger.write("ended at " + DateTimeFormatter.ISO_LOCAL_DATE_TIME.format(end) + "\n");
		logger.write("took " + Duration.between(start, end) + "\n");
		logger.close();
		System.out.println("started at " + DateTimeFormatter.ISO_LOCAL_DATE_TIME.format(start));
		System.out.println("ended at " + DateTimeFormatter.ISO_LOCAL_DATE_TIME.format(end));
		System.out.println("took " + Duration.between(start, end));
		System.out.println("done");
	}

	private static void compute(Path dataset, int i, int size, String type, Path dir, LogDiffer seed, List<Line> leftLines, List<Line> rightLines, Options options, BufferedWriter writer, BufferedWriter logger) throws IOException {
		String path = dataset.relativize(dir).toString();
		log(i, size, type, path);
		List<Long> durations = new ArrayList<>();
		Pair<List<Action>> actions = Pair.of(List.of(), List.of());
		int runs = 0;
		for (int loop = 0; loop < LOOPS; loop++) {
			runs += 1;
			long start = System.nanoTime();
			actions = callWithTimeout(() -> seed.diff(leftLines, rightLines, options));
//			actions = seed.diff(leftLines, rightLines, options);
			long end = System.nanoTime();
			long duration = end - start;
			durations.add(duration);
			if (actions.left().isEmpty() && actions.right().isEmpty()) {
				// the execution was stopped
				break;
			}
			if (duration >= 60 * 1_000_000_000.0) {
				// loop only once if the duration is greater than a minute
				break;
			}
		}
		if (actions.left().isEmpty() && actions.right().isEmpty()) {
			logrn(i, size, type, -1, path);
			writer.write(String.format(Locale.ENGLISH, "\"%s\",%s,%.1f,%d,%d,0,0,0,0,0,0,0,0,0,0%n",
					path, type, -1.0, leftLines.size(), rightLines.size()
			));
		} else {
			durations.sort(Long::compareTo);
			Metrics metrics = metric(actions);
			logrn(i, size, type, durations.get(runs / 2) / 1_000_000.0, path);
			writer.write(String.format(Locale.ENGLISH, "\"%s\",%s,%.1f,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d%n",
					path, type, (durations.get(runs / 2) / 1_000_000.0),
					leftLines.size(), rightLines.size(), metrics.actions, metrics.added, metrics.deleted,
					metrics.updated, metrics.movedUnchanged, metrics.movedUpdated,
					(metrics.similarBlockLeft + metrics.similarBlockRight), metrics.similarBlockLeft, metrics.similarBlockRight,
					runs
			));
		}
		writer.flush();
		logger.flush();
		// reset split lines cache
		Utils.resetCache();
	}

	public static Metrics metric(Pair<List<Action>> actions) {
		// count actions
		int[] counts = new int[5];
		counts[0] = (int) actions.right().stream().filter(a -> a.type() == Action.Type.ADDED).count();
		actions.left().forEach(a -> {
			switch (a.type()) {
				case DELETED -> counts[1]++;
				case UPDATED -> counts[2]++;
				case MOVED_UNCHANGED -> counts[3]++;
				case MOVED_UPDATED -> counts[4]++;
				case ADDED, UNCHANGED, NONE -> {
				}
			}
		});
		// count similar blocks
		int similarLeft = 1;
		if (!actions.left().isEmpty()) {
			Action.Type last = actions.left().get(0).type();
			for (Action action : actions.left()) {
				if (action.type() != last) {
					similarLeft++;
					last = action.type();
				}
			}
		} else {
			similarLeft = 0;
		}
		int similarRight = 1;
		if (!actions.right().isEmpty()) {
			Action.Type last = actions.right().get(0).type();
			for (Action action : actions.right()) {
				if (action.type() != last) {
					similarRight++;
					last = action.type();
				}
			}
		} else {
			similarRight = 0;
		}
		return new Metrics((int) (actions.left().stream().filter(a -> a.type() != Action.Type.UNCHANGED).count() + counts[0]),
				counts[0], counts[1], counts[2], counts[3], counts[4], similarLeft, similarRight);
	}

	public static List<Path> collectDirectories(Path dataset) {
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
			Files.walkFileTree(dataset, Set.of(FileVisitOption.FOLLOW_LINKS), 10, walker);
		} catch (IOException e) {
			throw new RuntimeException(e);
		}
		return directories;
	}

	public record Metrics(int actions, int added, int deleted, int updated, int movedUnchanged, int movedUpdated,
						  int similarBlockLeft, int similarBlockRight) {

	}

	public static DateTimeFormatter LOG_FORMAT = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss");

	public static void logrn(int i, int size, String algo, double duration, String logName) {
		long maxMemory = Runtime.getRuntime().maxMemory();
		long usedMemory = Runtime.getRuntime().totalMemory() - Runtime.getRuntime().freeMemory();
		// <timestamp> <log_index>/<logs_amount> (<percent>%) <algo> <log_name> <completion_duration> <memory_usage>
		System.out.printf("\r%s %5d/%d (%.1f%%) %-4s %-50s %.2fms %.2fMb/%.2fMb/%.2fMb%n", LOG_FORMAT.format(LocalDateTime.now()), i, size, i * 100.0 / size, algo, logName, duration, usedMemory / 1024.0 / 1024.0, Runtime.getRuntime().totalMemory() / 1024.0 / 1024.0, maxMemory / 1024.0 / 1024.0);
		try {
			logger.write("%s %5d/%d (%.1f%%) %-4s %-50s %.2fms %.2fMb/%.2fMb/%.2fMb%n".formatted(LOG_FORMAT.format(LocalDateTime.now()), i, size, i * 100.0 / size, algo, logName, duration, usedMemory / 1024.0 / 1024.0, Runtime.getRuntime().totalMemory() / 1024.0 / 1024.0, maxMemory / 1024.0 / 1024.0));
		} catch (IOException ignored) {
		}
	}

	public static void log(int i, int size, String algo, String logName) {
		long maxMemory = Runtime.getRuntime().maxMemory();
		long usedMemory = Runtime.getRuntime().totalMemory() - Runtime.getRuntime().freeMemory();
		// <timestamp> <log_index>/<logs_amount> (<percent>%) <algo> <log_name> <memory_usage>
		System.out.printf("%s %5d/%d (%.1f%%) %-4s %-50s %.2fMb/%.2fMb/%.2fMb", LOG_FORMAT.format(LocalDateTime.now()),
				i, size, i * 100.0 / size, algo, logName, usedMemory / 1024.0 / 1024.0, Runtime.getRuntime().totalMemory() / 1024.0 / 1024.0, maxMemory / 1024.0 / 1024.0);
		try {
			logger.write("%s %5d/%d (%.1f%%) %-4s %-50s %.2fMb/%.2fMb/%.2fMb%n".formatted(LOG_FORMAT.format(LocalDateTime.now()),
					i, size, i * 100.0 / size, algo, logName, usedMemory / 1024.0 / 1024.0, Runtime.getRuntime().totalMemory() / 1024.0 / 1024.0, maxMemory / 1024.0 / 1024.0));
		} catch (IOException ignored) {
		}
	}

}
