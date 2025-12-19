{{ config(materialized='view') }}

with source as (
    select * from {{ source('raw', 'products') }}
),

renamed as (
    select
        id as product_id,
        name as product_name,
        category,
        subcategory,
        price,
        cost,
        price - cost as gross_margin,
        case
            when cost > 0 then round((price - cost) / cost * 100, 2)
            else 0
        end as margin_pct,
        upper(sku) as sku,
        is_active,
        created_at
    from source
)

select * from renamed
