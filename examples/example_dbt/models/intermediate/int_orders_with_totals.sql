-- Calculates order totals from order items

with orders as (
    select * from {{ ref('stg_orders') }}
),

order_items as (
    select * from {{ ref('int_order_items_enriched') }}
),

order_totals as (
    select
        order_id,
        sum(quantity) as total_items,
        sum(line_total) as subtotal,
        sum(line_cost) as total_cost,
        sum(line_profit) as gross_profit,
        count(distinct product_id) as unique_products
    from order_items
    group by order_id
),

payments as (
    select
        order_id,
        sum(case when payment_status = 'completed' then payment_amount else 0 end) as total_paid,
        count(*) as payment_count
    from {{ ref('stg_payments') }}
    group by order_id
),

final as (
    select
        o.order_id,
        o.customer_id,
        o.order_date,
        o.order_status,
        o.shipping_country,
        o.discount_code,
        o.discount_amount,
        o.shipping_cost,
        coalesce(ot.total_items, 0) as total_items,
        coalesce(ot.unique_products, 0) as unique_products,
        coalesce(ot.subtotal, 0) as subtotal,
        coalesce(ot.subtotal, 0) - o.discount_amount + o.shipping_cost as order_total,
        coalesce(ot.total_cost, 0) as total_cost,
        coalesce(ot.gross_profit, 0) - o.discount_amount as net_profit,
        coalesce(p.total_paid, 0) as total_paid,
        coalesce(p.payment_count, 0) as payment_count,
        case
            when coalesce(ot.subtotal, 0) - o.discount_amount + o.shipping_cost <= coalesce(p.total_paid, 0)
            then true
            else false
        end as is_fully_paid,
        o.created_at
    from orders o
    left join order_totals ot on o.order_id = ot.order_id
    left join payments p on o.order_id = p.order_id
)

select * from final
