{{ config(materialized='table') }}

-- Payment method performance analysis

with payments as (
    select * from {{ ref('stg_payments') }}
),

orders as (
    select order_id, order_total
    from {{ ref('int_orders_with_totals') }}
),

payment_analysis as (
    select
        p.payment_method,
        count(*) as transaction_count,
        count(distinct p.order_id) as order_count,
        sum(p.payment_amount) as total_amount,
        round(avg(p.payment_amount), 2) as avg_transaction_amount,
        count(case when p.payment_status = 'completed' then 1 end) as successful_transactions,
        count(case when p.payment_status = 'pending' then 1 end) as pending_transactions,
        count(case when p.payment_status = 'refunded' then 1 end) as refunded_transactions,
        sum(case when p.payment_status = 'refunded' then p.payment_amount else 0 end) as refunded_amount
    from payments p
    group by p.payment_method
),

totals as (
    select
        sum(transaction_count) as total_transactions,
        sum(total_amount) as grand_total_amount
    from payment_analysis
)

select
    pa.payment_method,
    pa.transaction_count,
    pa.order_count,
    pa.total_amount,
    round(pa.total_amount / nullif(t.grand_total_amount, 0) * 100, 2) as pct_of_total_revenue,
    pa.avg_transaction_amount,
    pa.successful_transactions,
    pa.pending_transactions,
    pa.refunded_transactions,
    pa.refunded_amount,
    round(pa.successful_transactions::numeric / nullif(pa.transaction_count, 0) * 100, 2) as success_rate_pct,
    round(pa.refunded_amount / nullif(pa.total_amount, 0) * 100, 2) as refund_rate_pct
from payment_analysis pa
cross join totals t
order by pa.total_amount desc
