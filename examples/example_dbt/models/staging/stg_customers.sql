{{ config(materialized='view') }}

with source as (
    select * from {{ source('raw', 'customers') }}
),

renamed as (
    select
        id as customer_id,
        first_name,
        last_name,
        first_name || ' ' || last_name as full_name,
        lower(email) as email,
        phone,
        upper(country_code) as country_code,
        signup_source,
        created_at,
        updated_at
    from source
)

select * from renamed
