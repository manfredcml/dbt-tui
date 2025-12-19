{{ config(materialized='view') }}

with source as (
    select * from {{ source('raw', 'orders') }}
),

renamed as (
    select
        id as order_id,
        customer_id,
        order_date,
        status as order_status,
        upper(shipping_country) as shipping_country,
        discount_code,
        coalesce(discount_amount, 0) as discount_amount,
        coalesce(shipping_cost, 0) as shipping_cost,
        created_at
    from source
)

select * from renamed
