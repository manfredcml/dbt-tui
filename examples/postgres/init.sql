-- =============================================================================
-- E-commerce Analytics Database Initialization
-- =============================================================================

-- Create schemas for dbt
CREATE SCHEMA IF NOT EXISTS raw;
CREATE SCHEMA IF NOT EXISTS staging;
CREATE SCHEMA IF NOT EXISTS marts;
CREATE SCHEMA IF NOT EXISTS marts_finance;
CREATE SCHEMA IF NOT EXISTS marts_marketing;

-- Grant permissions
GRANT ALL ON SCHEMA raw TO dbt_user;
GRANT ALL ON SCHEMA staging TO dbt_user;
GRANT ALL ON SCHEMA marts TO dbt_user;
GRANT ALL ON SCHEMA marts_finance TO dbt_user;
GRANT ALL ON SCHEMA marts_marketing TO dbt_user;

-- =============================================================================
-- RAW TABLES
-- =============================================================================

-- Customers table
CREATE TABLE IF NOT EXISTS raw.customers (
    id INTEGER PRIMARY KEY,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    email VARCHAR(255),
    phone VARCHAR(50),
    country_code VARCHAR(3),
    signup_source VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Products table
CREATE TABLE IF NOT EXISTS raw.products (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255),
    category VARCHAR(100),
    subcategory VARCHAR(100),
    price DECIMAL(10, 2),
    cost DECIMAL(10, 2),
    sku VARCHAR(50),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Orders table
CREATE TABLE IF NOT EXISTS raw.orders (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER,
    order_date DATE,
    status VARCHAR(50),
    shipping_address TEXT,
    shipping_country VARCHAR(3),
    discount_code VARCHAR(50),
    discount_amount DECIMAL(10, 2) DEFAULT 0,
    shipping_cost DECIMAL(10, 2) DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Order items table
CREATE TABLE IF NOT EXISTS raw.order_items (
    id INTEGER PRIMARY KEY,
    order_id INTEGER,
    product_id INTEGER,
    quantity INTEGER,
    unit_price DECIMAL(10, 2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Payments table
CREATE TABLE IF NOT EXISTS raw.payments (
    id INTEGER PRIMARY KEY,
    order_id INTEGER,
    payment_method VARCHAR(50),
    amount DECIMAL(10, 2),
    status VARCHAR(50) DEFAULT 'completed',
    processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Marketing campaigns table
CREATE TABLE IF NOT EXISTS raw.campaigns (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255),
    channel VARCHAR(100),
    start_date DATE,
    end_date DATE,
    budget DECIMAL(12, 2),
    target_audience VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Customer campaign interactions
CREATE TABLE IF NOT EXISTS raw.campaign_interactions (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER,
    campaign_id INTEGER,
    interaction_type VARCHAR(50),
    interaction_date TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Product reviews
CREATE TABLE IF NOT EXISTS raw.reviews (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER,
    product_id INTEGER,
    order_id INTEGER,
    rating INTEGER,
    review_text TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- =============================================================================
-- INSERT SAMPLE DATA
-- =============================================================================

-- Customers (20 customers)
INSERT INTO raw.customers (id, first_name, last_name, email, phone, country_code, signup_source) VALUES
(1, 'John', 'Doe', 'john.doe@example.com', '+1-555-0101', 'US', 'organic'),
(2, 'Jane', 'Smith', 'jane.smith@example.com', '+1-555-0102', 'US', 'google_ads'),
(3, 'Bob', 'Johnson', 'bob.johnson@example.com', '+1-555-0103', 'US', 'referral'),
(4, 'Alice', 'Williams', 'alice.williams@example.com', '+44-555-0104', 'GB', 'organic'),
(5, 'Charlie', 'Brown', 'charlie.brown@example.com', '+1-555-0105', 'CA', 'facebook_ads'),
(6, 'Diana', 'Miller', 'diana.miller@example.com', '+49-555-0106', 'DE', 'google_ads'),
(7, 'Edward', 'Davis', 'edward.davis@example.com', '+1-555-0107', 'US', 'organic'),
(8, 'Fiona', 'Garcia', 'fiona.garcia@example.com', '+33-555-0108', 'FR', 'referral'),
(9, 'George', 'Martinez', 'george.martinez@example.com', '+1-555-0109', 'US', 'email'),
(10, 'Helen', 'Anderson', 'helen.anderson@example.com', '+61-555-0110', 'AU', 'organic'),
(11, 'Ivan', 'Taylor', 'ivan.taylor@example.com', '+1-555-0111', 'US', 'google_ads'),
(12, 'Julia', 'Thomas', 'julia.thomas@example.com', '+1-555-0112', 'US', 'facebook_ads'),
(13, 'Kevin', 'Jackson', 'kevin.jackson@example.com', '+44-555-0113', 'GB', 'organic'),
(14, 'Lisa', 'White', 'lisa.white@example.com', '+1-555-0114', 'US', 'referral'),
(15, 'Michael', 'Harris', 'michael.harris@example.com', '+1-555-0115', 'US', 'organic'),
(16, 'Nancy', 'Martin', 'nancy.martin@example.com', '+81-555-0116', 'JP', 'google_ads'),
(17, 'Oscar', 'Thompson', 'oscar.thompson@example.com', '+1-555-0117', 'US', 'email'),
(18, 'Patricia', 'Garcia', 'patricia.garcia@example.com', '+1-555-0118', 'US', 'organic'),
(19, 'Quinn', 'Robinson', 'quinn.robinson@example.com', '+44-555-0119', 'GB', 'facebook_ads'),
(20, 'Rachel', 'Clark', 'rachel.clark@example.com', '+1-555-0120', 'US', 'referral')
ON CONFLICT (id) DO NOTHING;

-- Products (15 products across categories)
INSERT INTO raw.products (id, name, category, subcategory, price, cost, sku, is_active) VALUES
(1, 'Wireless Headphones Pro', 'Electronics', 'Audio', 149.99, 75.00, 'ELEC-AUD-001', TRUE),
(2, 'USB-C Hub 7-in-1', 'Electronics', 'Accessories', 49.99, 22.00, 'ELEC-ACC-001', TRUE),
(3, 'Mechanical Keyboard RGB', 'Electronics', 'Peripherals', 129.99, 55.00, 'ELEC-PER-001', TRUE),
(4, 'Smart Watch Series 5', 'Electronics', 'Wearables', 299.99, 150.00, 'ELEC-WER-001', TRUE),
(5, 'Organic Cotton T-Shirt', 'Apparel', 'Tops', 34.99, 12.00, 'APRL-TOP-001', TRUE),
(6, 'Slim Fit Jeans', 'Apparel', 'Bottoms', 79.99, 28.00, 'APRL-BOT-001', TRUE),
(7, 'Running Shoes Pro', 'Apparel', 'Footwear', 119.99, 45.00, 'APRL-FTW-001', TRUE),
(8, 'Winter Jacket', 'Apparel', 'Outerwear', 189.99, 70.00, 'APRL-OUT-001', TRUE),
(9, 'Yoga Mat Premium', 'Sports', 'Fitness', 45.99, 18.00, 'SPRT-FIT-001', TRUE),
(10, 'Dumbbell Set 20kg', 'Sports', 'Weights', 89.99, 35.00, 'SPRT-WGT-001', TRUE),
(11, 'Coffee Maker Deluxe', 'Home', 'Kitchen', 79.99, 32.00, 'HOME-KIT-001', TRUE),
(12, 'Air Purifier HEPA', 'Home', 'Appliances', 199.99, 85.00, 'HOME-APP-001', TRUE),
(13, 'Desk Lamp LED', 'Home', 'Lighting', 39.99, 15.00, 'HOME-LGT-001', TRUE),
(14, 'Bluetooth Speaker Mini', 'Electronics', 'Audio', 29.99, 12.00, 'ELEC-AUD-002', TRUE),
(15, 'Laptop Stand Aluminum', 'Electronics', 'Accessories', 59.99, 25.00, 'ELEC-ACC-002', FALSE)
ON CONFLICT (id) DO NOTHING;

-- Orders (30 orders across different statuses)
INSERT INTO raw.orders (id, customer_id, order_date, status, shipping_country, discount_code, discount_amount, shipping_cost) VALUES
(1, 1, '2024-01-15', 'delivered', 'US', NULL, 0, 5.99),
(2, 1, '2024-02-20', 'delivered', 'US', 'SAVE10', 15.00, 5.99),
(3, 2, '2024-01-18', 'delivered', 'US', NULL, 0, 5.99),
(4, 3, '2024-02-10', 'shipped', 'US', NULL, 0, 5.99),
(5, 4, '2024-03-05', 'delivered', 'GB', NULL, 0, 12.99),
(6, 5, '2024-03-15', 'cancelled', 'CA', NULL, 0, 8.99),
(7, 6, '2024-03-20', 'delivered', 'DE', 'WELCOME20', 25.00, 15.99),
(8, 7, '2024-03-22', 'delivered', 'US', NULL, 0, 5.99),
(9, 8, '2024-04-01', 'processing', 'FR', NULL, 0, 12.99),
(10, 9, '2024-04-05', 'delivered', 'US', 'SPRING15', 20.00, 5.99),
(11, 10, '2024-04-10', 'shipped', 'AU', NULL, 0, 18.99),
(12, 11, '2024-04-15', 'delivered', 'US', NULL, 0, 5.99),
(13, 12, '2024-04-18', 'delivered', 'US', 'SAVE10', 12.00, 5.99),
(14, 13, '2024-04-20', 'returned', 'GB', NULL, 0, 12.99),
(15, 14, '2024-04-22', 'delivered', 'US', NULL, 0, 5.99),
(16, 15, '2024-04-25', 'delivered', 'US', NULL, 0, 5.99),
(17, 16, '2024-04-28', 'shipped', 'JP', NULL, 0, 22.99),
(18, 17, '2024-05-01', 'pending', 'US', NULL, 0, 5.99),
(19, 18, '2024-05-03', 'delivered', 'US', 'VIP25', 50.00, 0),
(20, 19, '2024-05-05', 'delivered', 'GB', NULL, 0, 12.99),
(21, 20, '2024-05-08', 'processing', 'US', NULL, 0, 5.99),
(22, 1, '2024-05-10', 'delivered', 'US', NULL, 0, 5.99),
(23, 2, '2024-05-12', 'delivered', 'US', NULL, 0, 5.99),
(24, 3, '2024-05-15', 'shipped', 'US', 'FLASH30', 30.00, 5.99),
(25, 5, '2024-05-18', 'delivered', 'CA', NULL, 0, 8.99),
(26, 7, '2024-05-20', 'pending', 'US', NULL, 0, 5.99),
(27, 9, '2024-05-22', 'delivered', 'US', NULL, 0, 5.99),
(28, 11, '2024-05-25', 'delivered', 'US', 'SAVE10', 18.00, 5.99),
(29, 13, '2024-05-28', 'processing', 'GB', NULL, 0, 12.99),
(30, 15, '2024-05-30', 'delivered', 'US', NULL, 0, 5.99)
ON CONFLICT (id) DO NOTHING;

-- Order items (multiple items per order)
INSERT INTO raw.order_items (id, order_id, product_id, quantity, unit_price) VALUES
(1, 1, 1, 1, 149.99),
(2, 2, 3, 1, 129.99),
(3, 2, 2, 2, 49.99),
(4, 3, 5, 2, 34.99),
(5, 3, 6, 1, 79.99),
(6, 4, 4, 1, 299.99),
(7, 5, 7, 1, 119.99),
(8, 5, 9, 1, 45.99),
(9, 6, 8, 1, 189.99),
(10, 7, 11, 1, 79.99),
(11, 7, 13, 2, 39.99),
(12, 8, 1, 1, 149.99),
(13, 8, 14, 2, 29.99),
(14, 9, 12, 1, 199.99),
(15, 10, 10, 1, 89.99),
(16, 10, 9, 2, 45.99),
(17, 11, 4, 1, 299.99),
(18, 12, 2, 1, 49.99),
(19, 12, 3, 1, 129.99),
(20, 13, 5, 3, 34.99),
(21, 14, 8, 1, 189.99),
(22, 15, 1, 1, 149.99),
(23, 16, 6, 2, 79.99),
(24, 16, 5, 1, 34.99),
(25, 17, 4, 1, 299.99),
(26, 18, 7, 1, 119.99),
(27, 19, 1, 1, 149.99),
(28, 19, 3, 1, 129.99),
(29, 19, 2, 1, 49.99),
(30, 20, 11, 1, 79.99),
(31, 21, 10, 2, 89.99),
(32, 22, 14, 1, 29.99),
(33, 23, 9, 1, 45.99),
(34, 24, 4, 1, 299.99),
(35, 25, 5, 2, 34.99),
(36, 25, 6, 1, 79.99),
(37, 26, 12, 1, 199.99),
(38, 27, 13, 3, 39.99),
(39, 28, 1, 1, 149.99),
(40, 28, 2, 1, 49.99),
(41, 29, 7, 2, 119.99),
(42, 30, 11, 1, 79.99),
(43, 30, 13, 1, 39.99)
ON CONFLICT (id) DO NOTHING;

-- Payments
INSERT INTO raw.payments (id, order_id, payment_method, amount, status, processed_at) VALUES
(1, 1, 'credit_card', 155.98, 'completed', '2024-01-15 10:30:00'),
(2, 2, 'paypal', 220.96, 'completed', '2024-02-20 14:45:00'),
(3, 3, 'credit_card', 155.96, 'completed', '2024-01-18 09:15:00'),
(4, 4, 'credit_card', 305.98, 'completed', '2024-02-10 16:20:00'),
(5, 5, 'bank_transfer', 178.97, 'completed', '2024-03-05 11:00:00'),
(6, 6, 'credit_card', 198.98, 'refunded', '2024-03-15 13:30:00'),
(7, 7, 'paypal', 150.96, 'completed', '2024-03-20 15:45:00'),
(8, 8, 'credit_card', 215.96, 'completed', '2024-03-22 10:00:00'),
(9, 9, 'credit_card', 212.98, 'pending', '2024-04-01 12:30:00'),
(10, 10, 'gift_card', 167.96, 'completed', '2024-04-05 14:00:00'),
(11, 11, 'paypal', 318.98, 'completed', '2024-04-10 09:30:00'),
(12, 12, 'credit_card', 185.97, 'completed', '2024-04-15 11:15:00'),
(13, 13, 'credit_card', 104.96, 'completed', '2024-04-18 16:45:00'),
(14, 14, 'bank_transfer', 202.98, 'refunded', '2024-04-20 10:30:00'),
(15, 15, 'credit_card', 155.98, 'completed', '2024-04-22 13:00:00'),
(16, 16, 'paypal', 200.96, 'completed', '2024-04-25 15:30:00'),
(17, 17, 'credit_card', 322.98, 'completed', '2024-04-28 09:00:00'),
(18, 18, 'credit_card', 125.98, 'pending', '2024-05-01 14:45:00'),
(19, 19, 'gift_card', 279.97, 'completed', '2024-05-03 11:30:00'),
(20, 20, 'credit_card', 92.98, 'completed', '2024-05-05 16:00:00'),
(21, 21, 'paypal', 185.97, 'pending', '2024-05-08 10:15:00'),
(22, 22, 'credit_card', 35.98, 'completed', '2024-05-10 13:45:00'),
(23, 23, 'credit_card', 51.98, 'completed', '2024-05-12 09:30:00'),
(24, 24, 'bank_transfer', 275.98, 'completed', '2024-05-15 15:00:00'),
(25, 25, 'paypal', 158.96, 'completed', '2024-05-18 11:00:00'),
(26, 26, 'credit_card', 205.98, 'pending', '2024-05-20 14:30:00'),
(27, 27, 'credit_card', 125.96, 'completed', '2024-05-22 10:45:00'),
(28, 28, 'gift_card', 187.97, 'completed', '2024-05-25 16:15:00'),
(29, 29, 'paypal', 252.97, 'pending', '2024-05-28 09:45:00'),
(30, 30, 'credit_card', 125.98, 'completed', '2024-05-30 13:00:00')
ON CONFLICT (id) DO NOTHING;

-- Marketing campaigns
INSERT INTO raw.campaigns (id, name, channel, start_date, end_date, budget, target_audience) VALUES
(1, 'Spring Sale 2024', 'email', '2024-03-01', '2024-03-31', 5000.00, 'all_customers'),
(2, 'Electronics Promo', 'google_ads', '2024-02-15', '2024-04-15', 15000.00, 'tech_enthusiasts'),
(3, 'New Customer Welcome', 'facebook_ads', '2024-01-01', '2024-12-31', 10000.00, 'new_signups'),
(4, 'Summer Collection', 'instagram', '2024-05-01', '2024-08-31', 8000.00, 'fashion_buyers'),
(5, 'Flash Sale Weekend', 'email', '2024-05-15', '2024-05-17', 2000.00, 'high_value_customers'),
(6, 'Fitness Month', 'google_ads', '2024-04-01', '2024-04-30', 6000.00, 'fitness_enthusiasts')
ON CONFLICT (id) DO NOTHING;

-- Campaign interactions
INSERT INTO raw.campaign_interactions (id, customer_id, campaign_id, interaction_type, interaction_date) VALUES
(1, 1, 1, 'email_open', '2024-03-05 09:00:00'),
(2, 1, 1, 'click', '2024-03-05 09:05:00'),
(3, 2, 2, 'impression', '2024-03-10 14:00:00'),
(4, 2, 2, 'click', '2024-03-10 14:02:00'),
(5, 3, 3, 'impression', '2024-02-15 10:00:00'),
(6, 4, 1, 'email_open', '2024-03-08 11:00:00'),
(7, 5, 3, 'impression', '2024-03-01 16:00:00'),
(8, 5, 3, 'click', '2024-03-01 16:01:00'),
(9, 6, 2, 'impression', '2024-03-15 09:30:00'),
(10, 6, 2, 'click', '2024-03-15 09:35:00'),
(11, 6, 2, 'conversion', '2024-03-20 15:45:00'),
(12, 7, 1, 'email_open', '2024-03-12 08:00:00'),
(13, 9, 5, 'email_open', '2024-05-15 10:00:00'),
(14, 9, 5, 'click', '2024-05-15 10:05:00'),
(15, 9, 5, 'conversion', '2024-04-05 14:00:00'),
(16, 11, 6, 'impression', '2024-04-05 12:00:00'),
(17, 11, 6, 'click', '2024-04-05 12:03:00'),
(18, 12, 4, 'impression', '2024-05-10 15:00:00'),
(19, 15, 6, 'impression', '2024-04-20 11:00:00'),
(20, 15, 6, 'click', '2024-04-20 11:02:00')
ON CONFLICT (id) DO NOTHING;

-- Product reviews
INSERT INTO raw.reviews (id, customer_id, product_id, order_id, rating, review_text) VALUES
(1, 1, 1, 1, 5, 'Excellent sound quality! Best headphones I''ve ever owned.'),
(2, 2, 5, 3, 4, 'Good quality cotton, very comfortable. Runs slightly large.'),
(3, 2, 6, 3, 5, 'Perfect fit, great material. Will buy more colors.'),
(4, 4, 7, 5, 5, 'Super comfortable for running. Great arch support.'),
(5, 6, 11, 7, 4, 'Makes great coffee. Wish it had a larger water reservoir.'),
(6, 7, 1, 8, 5, 'Love these headphones! Noise cancellation is amazing.'),
(7, 7, 14, 8, 3, 'Decent speaker for the price. Bass could be better.'),
(8, 10, 4, 11, 5, 'Best smartwatch I''ve used. Battery lasts all day.'),
(9, 12, 5, 13, 4, 'Nice fabric, good quality. Colors match the photos.'),
(10, 15, 1, 22, 5, 'Third pair I''ve bought. Never disappoints!')
ON CONFLICT (id) DO NOTHING;
