# `parallel`

`parallel limit N -> $results` runs named `task` blocks and stores a structured dictionary keyed by task name. Results are ordered deterministically by declaration order (the dictionary formatter also sorts keys).

```shelly
parallel limit 2 -> $results
    task build
        command make
    end
    task test
        command cargo test
    end
end
```

Task variable state is isolated and restored after each task. If a task fails, pending tasks are not launched; already-running tasks are waited for, and the command returns an aggregate result/error. Shelly does not cancel processes or create process groups, so a running child may outlive failure handling. Task bodies run concurrently with at most `limit` task-local workers. Results and errors are collected in declaration order. On the first observed failure, pending tasks are not launched; already-started tasks are joined. Shelly does not cancel running tasks or their child processes.

In dry-run mode no task executes. The parallel operation and each task are emitted to the audit stream, and `$results` is unavailable because no execution result exists.

Timeouts and `defer` use the normal enclosing interpreter scope. A timeout remains an error and deferred cleanup runs before the task result is collected.
