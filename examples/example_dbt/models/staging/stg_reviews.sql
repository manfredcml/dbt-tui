{{ config(materialized='view') }}

with source as (
    select * from {{ source('raw', 'reviews') }}
),

renamed as (
    select
        id as review_id,
        customer_id,
        product_id,
        order_id,
        rating,
        case
            when rating >= 4 then 'positive'
            when rating = 3 then 'neutral'
            else 'negative'
        end as sentiment,
        review_text,
        length(review_text) as review_length,
        created_at
    from source
)

select * from renamed
