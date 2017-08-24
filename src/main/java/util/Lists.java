package util;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import static java.util.Collections.unmodifiableList;
import static java.util.Optional.empty;

public class Lists {
    private Lists() {
        throw new UtilityClassInstantiatedException();
    }

    public static <T> Optional<T> head(List<T> xs) {
        return xs.isEmpty() ? empty() : Optional.of(xs.get(0));
    }

    public static <T> Optional<List<T>> tail(List<T> xs) {
        final Optional<List<T>> result;
        if (xs.isEmpty()) {
            result = empty();
        } else {
            List<T> newList = new ArrayList<>(xs);
            newList.remove(0);
            result = Optional.of(unmodifiableList(newList));
        }
        return result;
    }

    public static <T> List<T> cons(T x, List<T> xs) {
        List<T> ys = new ArrayList<>();
        ys.add(x);
        ys.addAll(xs);
        return unmodifiableList(ys);
    }
}
