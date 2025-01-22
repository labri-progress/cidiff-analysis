package com.github.cidiff.exp;

import org.github.cidiff.Action;
import org.github.cidiff.DrainData;
import org.github.cidiff.Line;
import org.github.cidiff.LogDiffer;
import org.github.cidiff.LogParser;
import org.github.cidiff.Metric;
import org.github.cidiff.Options;
import org.github.cidiff.Pair;
import org.github.cidiff.parsers.DrainParser;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.FileReader;
import java.io.FileWriter;
import java.io.IOException;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Stream;

public class Main {

	public static final String PATHS = "/home/ketheroth/these/projects/apollo/paths.txt";
	public static final String DATASET = "/home/ketheroth/these/datasets/moriconi";

	public static void main(String[] args) throws IOException {
		List<String> paths = new BufferedReader(new FileReader(PATHS)).lines().toList();
		LogParser parser = LogParser.Type.GITHUB.construct();
		LogParser parserDrain = LogParser.Type.DRAIN.construct();
		LogDiffer differSeed = LogDiffer.Algorithm.SEED.construct();
		LogDiffer differLCS = LogDiffer.Algorithm.LCS.construct();

		Options optSeed = new Options();
		Options optLCS = new Options().with(Options.METRIC, Metric.EQUALITY);
		Options optDrain = new Options().with(Options.METRIC, Metric.DRAIN_JOCKER_YES);

		BufferedWriter writer = new BufferedWriter(new FileWriter("result2.csv"));
		writer.write("path,type,line\n");
		for (String path : paths) {
			System.out.print(path + " ");
			List<Line> leftLines = parser.parse(DATASET + "/" + path + "/success.log", optSeed);
			List<Line> rightLines = parser.parse(DATASET + "/" + path + "/failure.log", optSeed);
			// DRAIN START
			DrainParser drain = new DrainParser(4, 0.5f, 100);
			drain.parse(Stream.concat(leftLines.stream().map(Line::value), rightLines.stream().map(Line::value)).toList());
			DrainData.setup(drain);
			// DRAIN END
			diff(leftLines, rightLines, optSeed, differSeed, path, "seed", writer);
			diff(leftLines, rightLines, optLCS, differLCS, path, "lcs", writer);
			diff(leftLines, rightLines, optDrain, differSeed, path, "drain", writer);
			bigramDiff(path, parserDrain, writer);
			System.out.println();
		}
		writer.close();
	}

	private static void bigramDiff(String path, LogParser parser, BufferedWriter writer) {
		System.out.print("bigram ");
		Options opt = new Options();
		List<Line> leftLines = parser.parse(DATASET + "/" + path + "/success.log", opt);
		List<Line> rightLines = parser.parse(DATASET + "/" + path + "/failure.log", opt);
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
					writer.write("%s,bigram,%d%n".formatted(path, line.index()));
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

}
