package util;

import java.util.*;

import static java.util.Objects.requireNonNull;

/**
 * Utilities for collections.
 */
public final class Collections {
    private Collections() {
        throw new UtilityClassInstantiatedException();
    }

    /**
     * Non-destructively update {@code xs} with {@code ys}, overwriting existing members in {@code xs}.
     *
     * @return The new result.
     */
    public static <T> Set<T> update(Set<T> xs, Set<T> ys) {
        Set<T> result = new HashSet<>();
        result.addAll(requireNonNull(xs));
        result.addAll(requireNonNull(ys));
        return java.util.Collections.unmodifiableSet(result);
    }

    /**
     * Non-destructively update {@code xs} with {@code ys}, overwriting existing members in {@code xs}.
     *
     * @return The new result.
     */
    public static <T> List<T> update(List<T> xs, List<T> ys) {
        List<T> result = new ArrayList<>();
        result.addAll(requireNonNull(xs));
        result.addAll(requireNonNull(ys));
        return java.util.Collections.unmodifiableList(result);
    }

    /**
     * Non-destructively update {@code xs} with {@code ys}, overwriting existing members in {@code xs}.
     *
     * @return The new result.
     */
    public static <K, V> Map<K, V> update(Map<K, V> xs, Map<K, V> ys) {
        HashMap<K, V> result = new HashMap<>();
        result.putAll(requireNonNull(xs));
        result.putAll(requireNonNull(ys));
        return java.util.Collections.unmodifiableMap(result);
    }
}
