This program demonstrates a silent signal stack overflow.

What it does:

Mmap anon memory region mem1 and fill it with '1'.
Spawn a new thread (triggering mmap of a new signal stack).
Mmap another anon memory region mem2 and fill it with '2'.

Raise a signal. In the signal handler a local array is created and filled with '7'.
There is a good chance that either mem1 or mem2 gets overwritten by that array.
We print the last value in both mem1 and mem2 before and after singal handler execution.

Expected behavior 
  - segfault in the signal handler.
  
Observed behavior 
  - on MacOS 10.15.3 mem1 gets partially overwritten (last val1 changes from 1 to 7)
  - on Linux mem2 gets partially overwritten (last val2 changes from 2 to 7)
  - program ends sucessfully (no segfault)
  
