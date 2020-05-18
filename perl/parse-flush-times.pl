#!/usr/bin/perl

use 5.016;
use warnings;
use autodie;

while (my$line = <>) {
        if (my($time, $bytes, $total_time, $ih_time) = $line =~ m{(\d\d\d\d-\d\d-\d\dT\d\d:\d\d:\d\d[.]\d\d\d\d).* (\d+) bytes in ([\d.]+) secs [(]ih ([\d.]+) secs[)]}) {
                my$qfs_time = $total_time - $ih_time;
                say "$bytes; $qfs_time; $ih_time";
        }
}
