#!/usr/bin/env perl

use strict;
use warnings;

use v5.10;

my %ofs_stats;
my %size_stats;
my %files_stats;
while (defined(my $l = <>)) {
        # fs_pread /test_compact/in/orbit/042/table.internal_backlinks.31456:compact@orbit3.408958.1556818559.9644.dat 128@4
        if (my($fpath, $size, $ofs) = $l =~ m{\sfs_pread\s+(/\S+)\s+(\d+)[@](\d+)}) {
                #say "-> $fpath, $pos, $size";
                #$size = int($size/1024)*1024;
                #$ofs = int($ofs/1024)*1024;
                $ofs_stats{$ofs}++;
                $size_stats{$size}++;
                $files_stats{$fpath} = $ofs if $files_stats{$fpath} < $ofs;
        }
}

say 'Offsets:';
for my$ofs(sort {$a <=> $b} keys%ofs_stats) {
        say "\tofs:$ofs; count:$ofs_stats{$ofs}";
}
say'';


say 'Sizes:';
for my$size(sort {$a <=> $b} keys%size_stats) {
        say "\tsize:$size; count:$size_stats{$size}";
}
say'';

say 'Files:';
for my$file(sort {$a cmp $b} keys%files_stats) {
        say "\tfile:$file; max-ofs:$files_stats{$file}";
}
say'';
