# minmips-assembler
minmipsのアセンブラ

fib.txt
```
        addi    $s0,$0,0
        addi    $s1,$0,1
        add     $s2,$s0,$s1
        add     $s3,$0,$0
        addi    $s4,$0,8
for:    beq     $s3,$s4,done
        add     $s2,$s0,$s1
        add     $s0,$0,$s1
        add     $s1,$0,$s2
        addi    $s3,$s3,1
        sw      $s2,80($0)
        j       for
done:   sw      $s2,84($0)
```

```
$ cargo run fib.txt
20100000
20110001
02119020
00009820
20140008
12740006
02119020
00118020
00128820
22730001
ac120050
08000005
ac120054
00000000
00000000
...
```
