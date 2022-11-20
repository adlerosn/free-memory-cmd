# free-memory-cmd

Runs a command if memory usage exceeds a certain threshould.

Memory usage can be RAM, SWAP, or both combined.

## Usage

```
free-memory-cmd <RAM|SWAP|COMB> <percentage> <command>
```

### Examples

* Reboot if using over 80% of SWAP.
    ```sh
    free-memory-cmd SWAP 80 reboot
    ```
* Check if RAM usage is over 60% within a shell script
    ```sh
    free-memory-cmd RAM 60 './my-memory-saving-measures.sh'
    ```
* Check if RAM usage is over 60% within a shell script
    ```sh
    if ! free-memory-cmd RAM 60 false ; then
        # do your "over 60% RAM usage" thing here
    fi
    ```

### Crontab
It's perfectly possible to write jobs like:
```cron
* * * * * free-memory-cmd COMB 50 reboot
```
