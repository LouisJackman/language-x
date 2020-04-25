# Matching in Switch and Select

* They both have cases which each match one or more patterns separated by
  commas.
* Both have `default` clauses as a fallback "match all" clause.
* `switch` is exhaustive: a compiler error happens if not all cases are covered.
* `select` is exhaustive for the selected type, but sends logs of non-matching types to the `noReceiver`
  task's mailbox (whose behaviour can be manually overridden). When `select`
  happens, all messages with non-matching types are logged to the noReceiver mailbox
  until a message with a matching type is encountered. If the mailbox is empty, it waits
  until messages are sent, repeating the same behaviour until a matching message
  is encountered.
* `select` blocks the current task until someone sends the process a message of
  the specified type with a match. `timeout` clauses are available.