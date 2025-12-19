{% macro generate_date_spine(start_date, end_date) %}
    with date_spine as (
        select
            generate_series(
                '{{ start_date }}'::date,
                '{{ end_date }}'::date,
                '1 day'::interval
            )::date as date_day
    )
    select * from date_spine
{% endmacro %}
