package util;

import java.util.Iterator;
import java.util.stream.Stream;
import java.util.stream.StreamSupport;

public final class Streams {
    private Streams() {
        throw new UtilityClassInstantiatedException();
    }

    public static <T, U> Stream<Pair<T, U>> zip(Stream<T> xs, Stream<U> ys) {
        Iterable<Pair<T, U>> zs = () -> new ZippedStreamIterator<>(xs.iterator(), ys.iterator());
        return StreamSupport.stream(zs.spliterator(), false);
    }

    private static final class ZippedStreamIterator<T, U> implements Iterator<Pair<T, U>> {
        private final Iterator<T> xs;
        private final Iterator<U> ys;

        ZippedStreamIterator(Iterator<T> xs, Iterator<U> ys) {
            this.xs = xs;
            this.ys = ys;
        }

        @Override
        public boolean hasNext() {
            return xs.hasNext() && ys.hasNext();
        }

        @Override
        public Pair<T, U> next() {
            return new Pair<>(xs.next(), ys.next());
        }
    }
}
