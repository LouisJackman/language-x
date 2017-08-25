package util;

import junit.framework.TestCase;

import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import static java.util.Arrays.asList;

public final class StreamsTest extends TestCase {

    public void test() {
        Stream<Integer> xs = Stream.of(10, 20, 30);
        Stream<Character> ys = Stream.of('a', 'g', 'd');
        List<Pair<Integer, Character>> zs = Streams.zip(xs, ys).collect(Collectors.toList());

        assertEquals(
                asList(
                        new Pair<>(10, 'a'),
                        new Pair<>(20, 'g'),
                        new Pair<>(30, 'd')
                ),
                zs
        );
    }
}
