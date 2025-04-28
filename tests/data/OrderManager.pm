package OrderManager;

use strict;
use warnings;
use DBI;
use HTML::Template;
use Email::Sender::Simple;
use Email::Simple;
use Email::Simple::Creator;
use JSON::XS;
use POSIX;
use List::Util qw(sum);
use Data::Dumper;

# Database connection details
my $dsn = "DBI:mysql:database=orders;host=localhost";
my $username = "order_user";
my $password = "secret";

sub generate_order_page {
    my ($order_id) = @_;
    
    my $template = HTML::Template->new(filename => 'templates/order.tmpl');
    my $order = get_order_details($order_id);
    
    $template->param(
        ORDER_ID => $order->{id},
        CUSTOMER_NAME => $order->{customer_name},
        ITEMS => $order->{items},
        TOTAL => calculate_total($order->{items})
    );
    
    return $template->output();
}

sub calculate_total {
    my ($items) = @_;
    return sum(map { $_->{price} * $_->{quantity} } @$items);
}

sub get_order_details {
    my ($order_id) = @_;
    
    my $dbh = DBI->connect($dsn, $username, $password) 
        or die "Can't connect to database: $DBI::errstr";
        
    my $sth = $dbh->prepare("SELECT * FROM orders WHERE id = ?");
    $sth->execute($order_id);
    
    my $order = $sth->fetchrow_hashref();
    $dbh->disconnect();
    
    return $order;
}

sub validate_order {
    my ($order) = @_;
    
    die "No items in order" unless @{$order->{items}};
    die "Invalid customer info" unless $order->{customer_name} && $order->{customer_email};
    
    foreach my $item (@{$order->{items}}) {
        die "Invalid item" unless $item->{id} && $item->{quantity} > 0;
    }
    
    return 1;
}

sub save_order {
    my ($order) = @_;
    
    validate_order($order);
    
    my $dbh = DBI->connect($dsn, $username, $password);
    my $sth = $dbh->prepare("INSERT INTO orders (customer_name, total) VALUES (?, ?)");
    
    $sth->execute($order->{customer_name}, calculate_total($order->{items}));
    my $order_id = $dbh->last_insert_id();
    
    save_order_items($dbh, $order_id, $order->{items});
    $dbh->disconnect();
    
    return $order_id;
}

sub save_order_items {
    my ($dbh, $order_id, $items) = @_;
    
    my $sth = $dbh->prepare("INSERT INTO order_items (order_id, item_id, quantity, price) VALUES (?, ?, ?, ?)");
    
    foreach my $item (@$items) {
        $sth->execute($order_id, $item->{id}, $item->{quantity}, $item->{price});
    }
}

sub send_order_confirmation {
    my ($order_id) = @_;
    
    my $order = get_order_details($order_id);
    my $email = create_confirmation_email($order);
    
    Email::Sender::Simple->send($email);
}

sub create_confirmation_email {
    my ($order) = @_;
    
    my $email = Email::Simple->create(
        header => [
            To => $order->{customer_email},
            From => 'orders@example.com',
            Subject => "Order Confirmation #" . $order->{id},
        ],
        body => generate_email_body($order),
    );
    
    return $email;
}

1; 