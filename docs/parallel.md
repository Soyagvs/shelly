# Run independent tasks with bounded parallelism

Use `parallel` to run named `task` blocks concurrently while limiting the number of task-local workers. Shelly stores structured results by task name and aggregates them deterministically.

## Quick path

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

`limit` must be a positive integer. The destination receives a dictionary keyed by task name; results and errors are collected in declaration order, and dictionary formatting sorts keys.

## Execution behavior

- Task state is isolated and restored after each task.
- At most `limit` tasks run at once.
- A task failure prevents pending tasks from launching.
- Tasks already started are waited for and joined.
- Shelly does not cancel running tasks or their child processes, and does not create process groups. A child may outlive failure handling.
- The operation returns an aggregate result or error after started tasks finish.

Timeouts and `defer` use the enclosing interpreter scope. A timeout remains an error, and deferred cleanup runs before that task result is collected.

## Dry-run behavior

In dry-run mode, no task executes. Shelly emits the parallel operation and each task to the audit stream, then marks `$results` unavailable because no execution result exists. See [Dry-run and audit logs](dry-run.md).

## Reference

| Item | Rule |
| --- | --- |
| Syntax | `parallel limit N -> $results` with named `task` blocks |
| Worker bound | No more than `N` task-local workers |
| Ordering | Declaration order for collection; sorted keys when formatted |
| Failure | Pending tasks are not launched; started tasks are joined |
| Cancellation | None for tasks or child processes |
| Result | Structured dictionary in normal execution; unavailable in dry-run |

## Related guides

- [Timeouts](timeouts.md) — inherited deadlines and cleanup.
- [Error handling](errors.md) — inspect and recover from failures.
- [Dry-run and audit logs](dry-run.md) — planned parallel actions.
