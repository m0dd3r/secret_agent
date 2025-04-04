package Calculator;

use strict;
use warnings;
use Math::Complex;
use List::Util qw(sum);

sub add {
    my ($a, $b) = @_;
    return $a + $b;
}

sub multiply {
    my ($a, $b) = @_;
    return $a * $b;
}

sub calculate_average {
    my @numbers = @_;
    return 0 if !@numbers;
    return sum(@numbers) / scalar(@numbers);
}

1; 