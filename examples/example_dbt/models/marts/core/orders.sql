{{ config(materialized='table') }}

-- Order fact table with complete details

with orders as (
    select * from {{ ref('int_orders_with_totals') }}
),

customers as (
    select * from {{ ref('stg_customers') }}
),

countries as (
    select * from {{ ref('countries') }}
),

final as (
    select
        o.order_id,
        o.customer_id,
        c.full_name as customer_name,
        c.email as customer_email,
        o.order_date,
        extract(dow from o.order_date) as order_day_of_week,
        extract(month from o.order_date) as order_month,
        extract(year from o.order_date) as order_year,
        o.order_status,
        o.shipping_country,
        co.country_name as shipping_country_name,
        co.region as shipping_region,
        o.discount_code,
        o.discount_amount,
        o.shipping_cost,
        o.total_items,
        o.unique_products,
        o.subtotal,
        o.order_total,
        o.total_cost,
        o.net_profit,
        o.total_paid,
        o.payment_count,
        o.is_fully_paid,

        -- Derived fields
        case
            when o.discount_code is not null then true
            else false
        end as has_discount,

        case
            when o.order_total >= 200 then 'Large'
            when o.order_total >= 100 then 'Medium'
            else 'Small'
        end as order_size_bucket,

        o.created_at
    from orders o
    left join customers c on o.customer_id = c.customer_id
    left join countries co on o.shipping_country = co.country_code
)

select * from final
