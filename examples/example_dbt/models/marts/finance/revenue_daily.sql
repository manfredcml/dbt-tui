{{ config(materialized='table') }}

-- Daily revenue aggregation for finance reporting

with orders as (
    select * from {{ ref('int_orders_with_totals') }}
    where order_status not in ('cancelled')
),

daily_revenue as (
    select
        order_date,
        count(*) as order_count,
        sum(total_items) as items_sold,
        sum(subtotal) as gross_revenue,
        sum(discount_amount) as total_discounts,
        sum(shipping_cost) as shipping_revenue,
        sum(order_total) as net_revenue,
        sum(total_cost) as total_cogs,
        sum(net_profit) as gross_profit,
        round(avg(order_total), 2) as avg_order_value,
        count(distinct customer_id) as unique_customers
    from orders
    group by order_date
)

select
    order_date,
    order_count,
    items_sold,
    gross_revenue,
    total_discounts,
    shipping_revenue,
    net_revenue,
    total_cogs,
    gross_profit,
    case
        when net_revenue > 0
        then round(gross_profit / net_revenue * 100, 2)
        else 0
    end as profit_margin_pct,
    avg_order_value,
    unique_customers,

    -- Running totals
    sum(net_revenue) over (order by order_date) as cumulative_revenue,
    sum(gross_profit) over (order by order_date) as cumulative_profit

from daily_revenue
order by order_date
