{{ config(materialized='view') }}

with source as (
    select * from {{ source('raw', 'payments') }}
),

renamed as (
    select
        id as payment_id,
        order_id,
        lower(payment_method) as payment_method,
        amount as payment_amount,
        lower(status) as payment_status,
        processed_at,
        created_at
    from source
)

select * from renamed
