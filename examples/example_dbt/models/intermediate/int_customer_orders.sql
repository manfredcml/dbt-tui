-- Aggregates customer order history

with orders as (
    select * from {{ ref('int_orders_with_totals') }}
),

customer_orders as (
    select
        customer_id,
        count(*) as total_orders,
        count(case when order_status = 'delivered' then 1 end) as delivered_orders,
        count(case when order_status = 'cancelled' then 1 end) as cancelled_orders,
        count(case when order_status = 'returned' then 1 end) as returned_orders,
        sum(total_items) as lifetime_items,
        sum(order_total) as lifetime_revenue,
        sum(net_profit) as lifetime_profit,
        avg(order_total) as avg_order_value,
        min(order_date) as first_order_date,
        max(order_date) as most_recent_order_date,
        max(order_date) - min(order_date) as customer_tenure_days
    from orders
    where order_status not in ('cancelled')
    group by customer_id
)

select * from customer_orders
