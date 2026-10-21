-- =============================================================================
-- SUPERNOVA LABS STUDIO - ENTERPRISE BACKEND SCHEMA
-- Services:
-- 1. Universal Booking Engine (Sports, Studio, Barbershop, Rental PS, Warnet, etc.)
-- 2. Unified POS, ERP, CRM & Keuangan
-- 3. Ordering In Table with QR Code (Dine-in Resto)
-- 4. Trial & Demo Inquiries
-- =============================================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =============================================================================
-- DOMAIN 1: UNIVERSAL BOOKING ENGINE
-- =============================================================================

CREATE TABLE IF NOT EXISTS booking_business_types (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    icon VARCHAR(50) DEFAULT 'clock',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS booking_resources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_type_id VARCHAR(50) NOT NULL REFERENCES booking_business_types(id),
    name VARCHAR(150) NOT NULL,
    category VARCHAR(100) NOT NULL,
    hourly_rate NUMERIC(14, 2) NOT NULL CHECK (hourly_rate >= 0),
    capacity INT NOT NULL DEFAULT 1,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS bookings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_id UUID NOT NULL REFERENCES booking_resources(id) ON DELETE CASCADE,
    booking_code VARCHAR(30) UNIQUE NOT NULL,
    customer_name VARCHAR(150) NOT NULL,
    customer_phone VARCHAR(30) NOT NULL,
    customer_email VARCHAR(150),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    duration_hours NUMERIC(4, 1) NOT NULL CHECK (duration_hours > 0),
    total_amount NUMERIC(14, 2) NOT NULL CHECK (total_amount >= 0),
    down_payment NUMERIC(14, 2) NOT NULL DEFAULT 0,
    booking_status VARCHAR(30) NOT NULL DEFAULT 'confirmed' CHECK (booking_status IN ('pending', 'confirmed', 'in_progress', 'completed', 'cancelled')),
    payment_status VARCHAR(30) NOT NULL DEFAULT 'unpaid' CHECK (payment_status IN ('unpaid', 'down_payment_paid', 'paid_full', 'refunded')),
    payment_method VARCHAR(50) DEFAULT 'qris',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_booking_period CHECK (end_time > start_time)
);

CREATE INDEX IF NOT EXISTS idx_bookings_resource_time ON bookings (resource_id, start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_bookings_code ON bookings (booking_code);

-- =============================================================================
-- DOMAIN 2: POS, ERP, CRM & KEUANGAN
-- =============================================================================

CREATE TABLE IF NOT EXISTS pos_outlets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(150) NOT NULL,
    address TEXT,
    phone VARCHAR(30),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS erp_categories (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS erp_products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    outlet_id UUID REFERENCES pos_outlets(id),
    sku VARCHAR(60) UNIQUE NOT NULL,
    name VARCHAR(200) NOT NULL,
    category_id VARCHAR(50) REFERENCES erp_categories(id),
    unit VARCHAR(30) NOT NULL DEFAULT 'pcs',
    buy_price NUMERIC(14, 2) NOT NULL DEFAULT 0 CHECK (buy_price >= 0),
    sell_price NUMERIC(14, 2) NOT NULL CHECK (sell_price >= 0),
    stock_quantity INT NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
    min_stock_alert INT NOT NULL DEFAULT 5,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS erp_inventory_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    change_type VARCHAR(30) NOT NULL CHECK (change_type IN ('stock_in', 'sale', 'adjustment', 'damage', 'return')),
    quantity_changed INT NOT NULL,
    previous_stock INT NOT NULL,
    new_stock INT NOT NULL,
    reference_id VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_customers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(150) NOT NULL,
    phone VARCHAR(30) UNIQUE NOT NULL,
    email VARCHAR(150),
    tier VARCHAR(30) NOT NULL DEFAULT 'bronze' CHECK (tier IN ('bronze', 'silver', 'gold', 'platinum')),
    loyalty_points INT NOT NULL DEFAULT 0 CHECK (loyalty_points >= 0),
    total_spent NUMERIC(14, 2) NOT NULL DEFAULT 0 CHECK (total_spent >= 0),
    last_visit TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS erp_employees (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    outlet_id UUID REFERENCES pos_outlets(id),
    name VARCHAR(150) NOT NULL,
    role VARCHAR(50) NOT NULL CHECK (role IN ('manager', 'cashier', 'kitchen', 'inventory', 'staff')),
    phone VARCHAR(30),
    status VARCHAR(30) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'on_leave', 'inactive')),
    shift VARCHAR(30) NOT NULL DEFAULT 'morning' CHECK (shift IN ('morning', 'middle', 'night')),
    attendance_status VARCHAR(30) NOT NULL DEFAULT 'present' CHECK (attendance_status IN ('present', 'absent', 'late', 'off')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pos_orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_number VARCHAR(50) UNIQUE NOT NULL,
    outlet_id UUID REFERENCES pos_outlets(id),
    customer_id UUID REFERENCES crm_customers(id),
    cashier_id UUID REFERENCES erp_employees(id),
    subtotal NUMERIC(14, 2) NOT NULL CHECK (subtotal >= 0),
    discount_amount NUMERIC(14, 2) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(14, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(14, 2) NOT NULL CHECK (total_amount >= 0),
    payment_method VARCHAR(50) NOT NULL DEFAULT 'cash' CHECK (payment_method IN ('cash', 'qris', 'debit', 'transfer', 'credit')),
    payment_status VARCHAR(30) NOT NULL DEFAULT 'completed' CHECK (payment_status IN ('pending', 'completed', 'refunded')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pos_order_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES pos_orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES erp_products(id),
    product_name VARCHAR(200) NOT NULL,
    quantity INT NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(14, 2) NOT NULL,
    unit_buy_price NUMERIC(14, 2) NOT NULL DEFAULT 0,
    subtotal NUMERIC(14, 2) NOT NULL
);

CREATE TABLE IF NOT EXISTS finance_journals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    outlet_id UUID REFERENCES pos_outlets(id),
    entry_type VARCHAR(20) NOT NULL CHECK (entry_type IN ('income', 'expense')),
    category VARCHAR(100) NOT NULL,
    amount NUMERIC(14, 2) NOT NULL CHECK (amount > 0),
    reference_order_id UUID REFERENCES pos_orders(id),
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- DOMAIN 3: ORDERING IN TABLE WITH QR CODE (RESTO & CAFE)
-- =============================================================================

CREATE TABLE IF NOT EXISTS resto_outlets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(150) NOT NULL,
    business_name VARCHAR(150) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS resto_tables (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resto_id UUID REFERENCES resto_outlets(id),
    table_number VARCHAR(30) NOT NULL,
    qr_token VARCHAR(64) UNIQUE NOT NULL,
    section_area VARCHAR(50) NOT NULL DEFAULT 'indoor' CHECK (section_area IN ('indoor', 'outdoor', 'vip', 'terrace', 'smoking')),
    capacity INT NOT NULL DEFAULT 4,
    status VARCHAR(30) NOT NULL DEFAULT 'empty' CHECK (status IN ('empty', 'occupied', 'ordered', 'calling_waiter', 'reserved')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS resto_menu_categories (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS resto_menu_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    category_id VARCHAR(50) NOT NULL REFERENCES resto_menu_categories(id),
    name VARCHAR(200) NOT NULL,
    description TEXT,
    price NUMERIC(14, 2) NOT NULL CHECK (price >= 0),
    image_url TEXT,
    available_variants JSONB DEFAULT '[]'::jsonb,
    is_available BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS resto_table_orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_code VARCHAR(40) UNIQUE NOT NULL,
    table_id UUID NOT NULL REFERENCES resto_tables(id),
    customer_name VARCHAR(100) DEFAULT 'Guest',
    status VARCHAR(30) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'cooking', 'served', 'completed', 'cancelled')),
    payment_method VARCHAR(50) NOT NULL DEFAULT 'qris' CHECK (payment_method IN ('qris', 'pay_at_cashier', 'ewallet')),
    payment_status VARCHAR(30) NOT NULL DEFAULT 'unpaid' CHECK (payment_status IN ('unpaid', 'paid')),
    subtotal NUMERIC(14, 2) NOT NULL DEFAULT 0,
    tax_service NUMERIC(14, 2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(14, 2) NOT NULL DEFAULT 0,
    customer_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS resto_table_order_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES resto_table_orders(id) ON DELETE CASCADE,
    menu_item_id UUID NOT NULL REFERENCES resto_menu_items(id),
    item_name VARCHAR(200) NOT NULL,
    quantity INT NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(14, 2) NOT NULL,
    selected_variant VARCHAR(150),
    special_notes TEXT,
    subtotal NUMERIC(14, 2) NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_resto_tables_qr ON resto_tables (qr_token);
CREATE INDEX IF NOT EXISTS idx_resto_table_orders_status ON resto_table_orders (status);

-- =============================================================================
-- DOMAIN 4: TRIAL & LIVE DEMO INQUIRIES
-- =============================================================================

CREATE TABLE IF NOT EXISTS trial_inquiries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_name VARCHAR(200) NOT NULL,
    business_type VARCHAR(100) NOT NULL,
    contact_name VARCHAR(150) NOT NULL,
    phone_wa VARCHAR(40) NOT NULL,
    email VARCHAR(150),
    interested_service VARCHAR(100) NOT NULL,
    estimated_outlets INT DEFAULT 1,
    current_pain_points TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'contacted', 'demo_scheduled', 'closed_won', 'closed_lost')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- DOMAIN 5: BUSINESS USER AUTHENTICATION & 1-MONTH TRIAL SERVICE ACCESS
-- =============================================================================

CREATE TABLE IF NOT EXISTS business_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(150) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(150) NOT NULL,
    business_name VARCHAR(150) NOT NULL,
    business_type VARCHAR(50) NOT NULL,
    phone_number VARCHAR(40) NOT NULL,
    city VARCHAR(100) DEFAULT 'Indonesia',
    service_plan VARCHAR(50) NOT NULL DEFAULT 'trial_1_month',
    trial_starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    trial_expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_business_users_email ON business_users (email);

-- =============================================================================
-- SEED DATA (INITIAL SETUP FOR IMMEDIATE OUT-OF-THE-BOX USE)
-- =============================================================================

-- Seed Business Types for Booking
INSERT INTO booking_business_types (id, name, description, icon) VALUES
    ('sports', 'Sarana Olahraga', 'Futsal, Badminton, Basket, Mini Soccer', 'globe'),
    ('studio', 'Studio Photo & Production', 'Thematic Room, Self-Photo, Green Screen', 'spark'),
    ('barber', 'Barbershop & Salon', 'Pangkas Rambut, Coloring, Treatment', 'scissors'),
    ('rental_ps', 'Rental PlayStation', 'Console PS5/PS4, VIP Sofa, 4K Display', 'terminal'),
    ('warnet', 'Warnet & PC Gaming Cafe', 'High-end PC Gaming RTX, Billing Timer', 'desktop'),
    ('general_services', 'Layanan & Jasa Lainnya', 'Klinik, Cuci Mobil, Bimbingan Belajar', 'layers')
ON CONFLICT (id) DO NOTHING;

-- Seed Sample Booking Resources
INSERT INTO booking_resources (id, business_type_id, name, category, hourly_rate, capacity, description) VALUES
    ('11111111-1111-1111-1111-111111111111', 'sports', 'Lapangan Futsal Vinyl Pro 1', 'Vinyl Interlock FIFA Standard', 150000.00, 10, 'Lengkap dengan papan skor digital dan penerangan LED 800W'),
    ('22222222-2222-2222-2222-222222222222', 'sports', 'Lapangan Badminton Court A', 'Karpet Yonex Tournament', 65000.00, 4, 'Bebas angin, sirkulasi udara exhaust dingin'),
    ('33333333-3333-3333-3333-333333333333', 'studio', 'Studio 01 - Thematic Korean Minimalist', 'Thematic Room', 120000.00, 6, 'Dilengkapi Godox strobe lighting & properti estetik lengkap'),
    ('44444444-4444-4444-4444-444444444444', 'barber', 'Kursi Barber Executive 01', 'Stylist Senior', 60000.00, 1, 'Layanan potong, cuci, hot towel, dan pomade styling'),
    ('55555555-5555-5555-5555-555555555555', 'rental_ps', 'Station PS5 VIP Lounge 03', 'PlayStation 5 4K 120Hz', 35000.00, 4, 'TV OLED 55 Inch, 4 Stik DualSense, Sofa Recliner'),
    ('66666666-6666-6666-6666-666666666666', 'warnet', 'Gaming PC VIP #12', 'RTX 4070 32GB RAM 240Hz', 15000.00, 1, 'Kursi Secretlab, Mechanical Keyboard, Headset HyperX')
ON CONFLICT DO NOTHING;

-- Seed Sample Categories & Products for POS / ERP
INSERT INTO erp_categories (id, name) VALUES
    ('fnb_beans', 'Biji Kopi & Bahan Baku'),
    ('fnb_beverage', 'Minuman Jadi & Kopi'),
    ('fnb_food', 'Makanan & Snack'),
    ('merchandise', 'Merchandise & Retail')
ON CONFLICT (id) DO NOTHING;

INSERT INTO erp_products (id, sku, name, category_id, buy_price, sell_price, stock_quantity, min_stock_alert) VALUES
    ('aaaa1111-1111-1111-1111-111111111111', 'BEANS-SIG-250', 'Biji Kopi Signature Roast 250g', 'fnb_beans', 38000.00, 75000.00, 48, 10),
    ('aaaa2222-2222-2222-2222-222222222222', 'BEV-CARAMEL-MAC', 'Caramel Macchiato Latte', 'fnb_beverage', 11500.00, 35000.00, 92, 15),
    ('aaaa3333-3333-3333-3333-333333333333', 'BEV-MATCHA-ART', 'Artisan Uji Matcha Latte', 'fnb_beverage', 13000.00, 38000.00, 65, 10),
    ('aaaa4444-4444-4444-4444-444444444444', 'FOOD-WAGYU-BOWL', 'Wagyu Beef Donburi Rice Bowl', 'fnb_food', 42000.00, 88000.00, 34, 8)
ON CONFLICT DO NOTHING;

-- Seed Sample CRM Customers
INSERT INTO crm_customers (id, name, phone, email, tier, loyalty_points, total_spent) VALUES
    ('bbbb1111-1111-1111-1111-111111111111', 'Budi Santoso', '081298761234', 'budi.santoso@example.com', 'gold', 420, 2450000.00),
    ('bbbb2222-2222-2222-2222-222222222222', 'Siti Rahmawati', '081387654321', 'siti.rahma@example.com', 'silver', 180, 1150000.00)
ON CONFLICT DO NOTHING;

-- Seed Sample Employees
INSERT INTO erp_employees (id, name, role, phone, status, shift, attendance_status) VALUES
    ('cccc1111-1111-1111-1111-111111111111', 'Andi Wijaya', 'manager', '081122334455', 'active', 'morning', 'present'),
    ('cccc2222-2222-2222-2222-222222222222', 'Dewi Lestari', 'cashier', '081233445566', 'active', 'morning', 'present'),
    ('cccc3333-3333-3333-3333-333333333333', 'Chef Rahmat', 'kitchen', '081344556677', 'active', 'morning', 'present')
ON CONFLICT DO NOTHING;

-- Seed Sample Resto Tables & Menu for QR Dine-in
INSERT INTO resto_menu_categories (id, name, sort_order) VALUES
    ('main_course', 'Makanan Utama', 1),
    ('beverage', 'Minuman Segar & Kopi', 2),
    ('appetizer', 'Camilan & Dessert', 3)
ON CONFLICT (id) DO NOTHING;

INSERT INTO resto_tables (id, table_number, qr_token, section_area, capacity, status) VALUES
    ('dddd1111-1111-1111-1111-111111111111', '01', 'tbl_tok_01_supernova_demo', 'indoor', 4, 'empty'),
    ('dddd2222-2222-2222-2222-222222222222', '05', 'tbl_tok_05_outdoor_terrace', 'outdoor', 4, 'empty'),
    ('dddd3333-3333-3333-3333-333333333333', '08', 'tbl_tok_08_vip_room_lounge', 'vip', 8, 'occupied')
ON CONFLICT DO NOTHING;

INSERT INTO resto_menu_items (id, category_id, name, description, price, available_variants) VALUES
    ('eeee1111-1111-1111-1111-111111111111', 'main_course', 'Wagyu Beef Donburi Rice Bowl', 'Daging Wagyu Meltique empuk dengan saus tare gurih, onsen egg, dan taburan daun bawang Jepang.', 88000.00, '[{"name": "Tingkat Kematangan", "options": ["Medium", "Medium Well", "Well Done"]}, {"name": "Pedas", "options": ["Original", "Sedang", "Sangat Pedas"]}]'::jsonb),
    ('eeee2222-2222-2222-2222-222222222222', 'beverage', 'Iced Peach Tea Yakult', 'Teh peach harum berpadu dengan probiotik Yakult segar dan popping boba peach.', 28000.00, '[{"name": "Gula", "options": ["Normal Sugar", "Less Sugar", "No Sugar"]}, {"name": "Es", "options": ["Normal Ice", "Less Ice"]}]'::jsonb),
    ('eeee3333-3333-3333-3333-333333333333', 'appetizer', 'Truffle Parmesan Fries', 'Kentang goreng renyah dibalut minyak truffle putih premium dan keju parmesan parut segar.', 36000.00, '[]'::jsonb)
ON CONFLICT DO NOTHING;

-- Seed Sample Business User (1-Month Real Trial Active)
INSERT INTO business_users (
    id, email, password_hash, full_name, business_name, business_type, phone_number, city,
    service_plan, trial_starts_at, trial_expires_at, is_active
) VALUES (
    '99999999-9999-9999-9999-999999999999',
    'owner@supernovasports.id',
    'ef51a37c449339739502b48981467493e81a32a6a57519cf2e259e86ba6f98ef',
    'Rahmat Hidayat',
    'Supernova Arena Sports & Studio',
    'sports',
    '081234567890',
    'Jakarta Selatan',
    'trial_1_month',
    NOW(),
    NOW() + INTERVAL '30 days',
    TRUE
) ON CONFLICT (email) DO NOTHING;
