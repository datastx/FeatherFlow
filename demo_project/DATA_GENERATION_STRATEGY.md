# FeatherFlow Financial Demo - Data Generation Strategy

This document outlines the detailed strategy for generating realistic financial time-series data for the FeatherFlow demo, with a particular focus on creating believable transaction patterns across various time dimensions.

## Core Principles for Time-Series Generation

The data generation process will adhere to these core principles:

1. **Deterministic Reproducibility**: Using fixed random seeds to ensure the same dataset can be regenerated consistently
2. **Statistical Realism**: Distribution patterns matching real-world financial behaviors
3. **Temporal Coherence**: Logical relationships between transactions across time
4. **Pattern Variability**: Mixture of regular patterns and realistic irregularities
5. **Causal Relationships**: Dependencies between transaction types and timing

## Transaction Types and Their Temporal Patterns

Each transaction type will have distinct temporal characteristics:

### 1. Income Deposits

```
Pattern: Regular periodic deposits with slight variations
Frequency: Bi-weekly or monthly (based on customer profile)
Timing Distribution: Concentrated around 1st and 15th of month
Variations:
  - Amount: Small variance (±2-3%)
  - Timing: ±1 day from expected date
  - Occasional bonuses (quarterly pattern)
```

**Implementation Approach**:
```rust
// For each customer account flagged for income deposits
let deposit_day = if customer.pay_frequency == "bi-weekly" {
    vec![customer_seed % 5 + 1, customer_seed % 5 + 15]  // Slight variation per customer
} else {
    vec![customer_seed % 5 + 1]  // Monthly on ~1st
};

// For each month in simulation period
for month in start_date.month()..=end_date.month() {
    for day in &deposit_day {
        // Base amount with small variance
        let amount = customer.base_income * 
            (1.0 + normal_distribution.sample(&mut rng) * 0.02);
            
        // Date with slight variation
        let date = get_date(year, month, *day) + 
            Duration::days(binomial_distribution.sample(&mut rng) - 1);
            
        // Quarterly bonus (if applicable)
        if month % 3 == 0 && uniform_distribution.sample(&mut rng) < 0.7 {
            // Add bonus transaction
            let bonus_amount = customer.base_income * 
                uniform_distribution.sample(&mut rng) * 0.2;
            add_transaction(date + Duration::days(1), bonus_amount, "Bonus");
        }
        
        add_transaction(date, amount, "Salary Deposit");
    }
}
```

### 2. Recurring Bills

```
Pattern: Monthly fixed expenses
Frequency: Monthly, with specific day of month
Timing: Different for each bill type
  - Rent/Mortgage: 1st-5th
  - Utilities: 10th-20th
  - Subscriptions: Distributed throughout month
Variations:
  - Amount: Fixed for subscriptions, seasonal variation for utilities
  - Small date variance (±2 days)
```

**Implementation Approach**:
```rust
// For each account, generate set of recurring bills
let recurring_bills = generate_recurring_bills(customer);

// For each month in simulation period
for month in start_date.month()..=end_date.month() {
    for bill in &recurring_bills {
        let base_amount = bill.amount;
        
        // Apply seasonal adjustments for utilities
        let amount = if bill.category == "Utilities" {
            apply_seasonal_adjustment(base_amount, month, rng)
        } else {
            base_amount
        };
        
        // Date with small variance
        let date = get_date(year, month, bill.day) + 
            Duration::days(binomial_small_distribution.sample(&mut rng) - 1);
            
        add_transaction(date, -amount, bill.description);
    }
}

// Function to apply seasonal adjustments to utility bills
fn apply_seasonal_adjustment(amount: f64, month: u32, rng: &mut StdRng) -> f64 {
    // Higher in summer and winter months
    let seasonal_factor = match month {
        12 | 1 | 2 => 1.3,  // Winter: heating
        6 | 7 | 8 => 1.2,   // Summer: cooling
        _ => 1.0,           // Spring/Fall: moderate
    };
    
    // Add some noise (±5%)
    amount * seasonal_factor * (0.95 + uniform_distribution.sample(rng) * 0.1)
}
```

### 3. Daily/Weekly Variable Expenses

```
Pattern: Irregular expenses with day-of-week and time-of-day patterns
Frequency: Multiple times per week
Timing: 
  - Weekday lunch expenses (11am-1pm)
  - Weekend entertainment (evening, higher on Friday/Saturday)
  - Weekend shopping (daytime)
  - Grocery shopping (evenings, higher on Sunday)
Variations:
  - Amount follows log-normal distribution
  - Frequency varies by customer profile (income bracket)
```

**Implementation Approach**:
```rust
// Define probability weights for each day of week (0 = Sunday)
let day_weights = match transaction_category {
    "Dining" => [0.08, 0.14, 0.14, 0.14, 0.16, 0.20, 0.14],
    "Coffee" => [0.05, 0.18, 0.18, 0.18, 0.18, 0.15, 0.08],
    "Groceries" => [0.25, 0.10, 0.10, 0.10, 0.15, 0.15, 0.15],
    "Entertainment" => [0.05, 0.02, 0.02, 0.05, 0.25, 0.50, 0.11],
    _ => [0.14, 0.14, 0.14, 0.14, 0.14, 0.15, 0.15],
};

// Time of day weights (0=morning, 1=afternoon, 2=evening, 3=night)
let time_weights = match transaction_category {
    "Dining" => [0.15, 0.45, 0.35, 0.05],
    "Coffee" => [0.60, 0.30, 0.05, 0.05], 
    "Groceries" => [0.20, 0.30, 0.45, 0.05],
    "Entertainment" => [0.05, 0.25, 0.60, 0.10],
    _ => [0.25, 0.25, 0.25, 0.25],
};

// Customer spending frequency based on income bracket
let weekly_frequency = match customer.income_bracket {
    "High" => poisson_distribution(12.0).sample(&mut rng),
    "Medium" => poisson_distribution(8.0).sample(&mut rng),
    "Low" => poisson_distribution(5.0).sample(&mut rng),
};

// Generate transactions for each week
for week_start in each_week_start(start_date, end_date) {
    // Number of transactions this week (with some variation)
    let transaction_count = (weekly_frequency as f64 * 
        normal_distribution(1.0, 0.2).sample(&mut rng)).max(0.0).round() as usize;
    
    for _ in 0..transaction_count {
        // Select day of week based on category weights
        let day_of_week = weighted_choice(&day_weights, &mut rng);
        
        // Select time of day based on category weights
        let time_of_day = weighted_choice(&time_weights, &mut rng);
        
        // Generate transaction date/time
        let transaction_date = week_start + Duration::days(day_of_week as i64);
        let transaction_time = generate_time_for_period(time_of_day, &mut rng);
        let transaction_datetime = combine_date_time(transaction_date, transaction_time);
        
        // Amount based on log-normal distribution (category and income specific)
        let amount = -generate_amount_for_category(
            transaction_category, 
            customer.income_bracket, 
            &mut rng
        );
        
        // Select merchant based on category
        let merchant = select_merchant_for_category(transaction_category, &mut rng);
        
        add_detailed_transaction(
            transaction_datetime, 
            amount, 
            transaction_category,
            merchant.id,
            day_of_week,
            false  // not recurring
        );
    }
}
```

### 4. Seasonal Spending Patterns

```
Pattern: Category-specific seasonal variation
Frequency: Annual cycles
Examples:
  - Holiday shopping (November-December)
  - Summer vacation spending (June-August)
  - Back-to-school (August-September)
  - Tax-related (January, April)
Variations:
  - Amount and frequency increases during relevant seasons
  - Merchant categories shift seasonally
```

**Implementation Approach**:
```rust
// Define seasonal multipliers for spending categories by month (1-indexed)
let seasonal_multipliers = {
    "Travel": [0.7, 0.7, 0.9, 1.0, 1.1, 1.4, 1.8, 1.7, 1.0, 0.8, 0.7, 0.9],
    "Gifts": [0.6, 0.8, 0.7, 0.6, 1.0, 0.8, 0.7, 0.7, 0.8, 0.9, 1.5, 2.5],
    "Clothing": [1.0, 0.9, 1.0, 1.1, 1.1, 1.0, 1.1, 1.5, 1.3, 1.0, 1.0, 1.2],
    "Electronics": [0.8, 0.8, 0.9, 0.9, 1.0, 0.9, 0.9, 1.0, 1.1, 1.0, 1.8, 2.0],
    "Education": [0.7, 0.7, 0.8, 0.8, 0.9, 1.0, 1.0, 2.0, 1.8, 1.0, 0.8, 0.7],
};

// Generate seasonal spending for each month
for month in 1..=12 {
    for category in SEASONAL_CATEGORIES {
        let multiplier = seasonal_multipliers[category][month-1];
        
        // Only generate transactions if multiplier is significant
        if multiplier > 0.5 {
            // Number of transactions this month, influenced by seasonal factor
            let base_count = match customer.income_bracket {
                "High" => 3,
                "Medium" => 2, 
                "Low" => 1,
            };
            
            let transaction_count = (base_count as f64 * 
                multiplier * 
                normal_distribution(1.0, 0.3).sample(&mut rng)).round() as usize;
            
            for _ in 0..transaction_count {
                // Generate transaction details with seasonal influence
                let day = uniform_int_distribution(1, 28).sample(&mut rng);
                let amount = -generate_seasonal_amount(
                    category, 
                    month, 
                    customer.income_bracket,
                    &mut rng
                );
                
                let date = get_date(year, month, day);
                add_transaction(date, amount, category);
            }
        }
    }
}
```

### 5. Major Life Events (Infrequent Large Transactions)

```
Pattern: Rare, high-value transactions
Frequency: Infrequent (a few per year)
Examples: 
  - Large purchases (electronics, furniture)
  - Medical expenses
  - Car repairs
  - Travel bookings
Variations:
  - Heavily right-skewed distribution
  - Often followed by related smaller transactions
```

**Implementation Approach**:
```rust
// Probability of major event per month based on customer profile
let monthly_major_event_probability = match customer.income_bracket {
    "High" => 0.15,
    "Medium" => 0.08,
    "Low" => 0.05,
};

// Categories with typical amounts
let major_event_types = [
    ("Major Purchase", 500.0, 5000.0),
    ("Medical Expense", 200.0, 3000.0),
    ("Car Repair", 300.0, 2000.0),
    ("Travel Booking", 500.0, 3000.0),
    ("Home Improvement", 300.0, 4000.0),
];

// For each month in simulation period
for month in start_date.month()..=end_date.month() {
    // Check if major event occurs this month
    if uniform_distribution.sample(&mut rng) < monthly_major_event_probability {
        // Select event type
        let event_index = uniform_int_distribution(0, major_event_types.len() - 1)
            .sample(&mut rng);
        let (event_type, min_amount, max_amount) = major_event_types[event_index];
        
        // Generate amount (log-normal to skew toward lower end of range)
        let amount = generate_skewed_amount(min_amount, max_amount, &mut rng);
        
        // Pick a day in the month
        let day = uniform_int_distribution(1, 28).sample(&mut rng);
        let date = get_date(year, month, day);
        
        add_transaction(date, -amount, event_type);
        
        // Generate follow-up transactions
        if uniform_distribution.sample(&mut rng) < 0.7 {
            let follow_up_count = poisson_distribution(2.0).sample(&mut rng);
            for i in 1..=follow_up_count {
                let follow_up_date = date + Duration::days(
                    uniform_int_distribution(1, 14).sample(&mut rng)
                );
                let follow_up_amount = amount * 
                    uniform_distribution.sample(&mut rng) * 0.2;
                    
                add_transaction(
                    follow_up_date, 
                    -follow_up_amount, 
                    format!("{} - Related", event_type)
                );
            }
        }
    }
}
```

## Generating Time-Series Coherence

To ensure realistic patterns across time dimensions, we'll implement several coherence mechanisms:

### 1. Balance Maintenance

```rust
// Track running balance for each account
let mut current_balance = account.initial_balance;

// Sort transactions chronologically
transactions.sort_by(|a, b| a.datetime.cmp(&b.datetime));

// Ensure balance never goes too negative
for transaction in &mut transactions {
    current_balance += transaction.amount;
    
    // If balance goes too negative, insert a deposit
    if current_balance < -account.overdraft_limit {
        let deposit_amount = (-current_balance) + 
            account.initial_balance * uniform_distribution.sample(&mut rng) * 0.5;
            
        let deposit_date = transaction.date + Duration::days(
            uniform_int_distribution(1, 3).sample(&mut rng)
        );
        
        add_transaction(deposit_date, deposit_amount, "Transfer Deposit");
        current_balance += deposit_amount;
    }
}
```

### 2. Cyclical Income and Expense Correlation

```rust
// Track income dates to correlate large expenses
let mut income_dates = Vec::new();

// First pass: generate income transactions and track dates
for month in start_date.month()..=end_date.month() {
    // Generate income deposits as described earlier
    let deposit_date = /* income calculation */;
    income_dates.push(deposit_date);
}

// Second pass: schedule large expenses near income dates
for &income_date in &income_dates {
    if uniform_distribution.sample(&mut rng) < 0.4 {
        // Schedule large expense shortly after income
        let days_after = uniform_int_distribution(1, 5).sample(&mut rng);
        let expense_date = income_date + Duration::days(days_after);
        let expense_amount = customer.base_income * 
            uniform_distribution.sample(&mut rng) * 0.3;
            
        add_transaction(expense_date, -expense_amount, "Major Expense");
    }
}
```

### 3. Consistent Merchant Relationships

```rust
// Create customer-merchant affinity map
let mut merchant_affinity = HashMap::new();

// Assign affinity scores to merchants based on customer profile
for merchant in merchants {
    let affinity = calculate_affinity(customer, merchant, &mut rng);
    merchant_affinity.insert(merchant.id, affinity);
}

// When generating transactions, weight merchant selection by affinity
fn select_merchant_for_category(category: &str, rng: &mut StdRng) -> Merchant {
    let category_merchants = merchants_by_category[category];
    
    // Create weighted distribution based on affinity
    let weights: Vec<f64> = category_merchants.iter()
        .map(|m| merchant_affinity[m.id])
        .collect();
        
    let dist = WeightedIndex::new(&weights).unwrap();
    let selected_index = dist.sample(rng);
    
    category_merchants[selected_index].clone()
}
```

### 4. Realistic Time-of-Day Patterns

```rust
fn generate_time_for_period(period: usize, rng: &mut StdRng) -> NaiveTime {
    let (start_hour, end_hour) = match period {
        0 => (6, 11),   // Morning
        1 => (11, 16),  // Afternoon
        2 => (16, 21),  // Evening
        3 => (21, 24),  // Night
        _ => (9, 18),   // Default (business hours)
    };
    
    // Generate hour (weighted toward middle of period)
    let hour_dist = create_triangular_distribution(start_hour, end_hour, rng);
    let hour = hour_dist.sample(rng);
    
    // Generate minute (uniform)
    let minute = uniform_int_distribution(0, 59).sample(rng);
    
    NaiveTime::from_hms(hour, minute, 0)
}
```

## Ensuring Statistical Realism

To create realistic distributions of transaction amounts, we'll use appropriate statistical distributions:

### 1. Regular Expenses

```rust
// For regular expenses like dining, groceries, etc.
fn generate_amount_for_category(
    category: &str, 
    income_bracket: &str,
    rng: &mut StdRng
) -> f64 {
    // Parameters based on category and income bracket
    let (mu, sigma) = match (category, income_bracket) {
        ("Dining", "High") => (4.2, 0.5),     // ~$75 median
        ("Dining", "Medium") => (3.6, 0.4),   // ~$35 median
        ("Dining", "Low") => (3.0, 0.3),      // ~$20 median
        ("Groceries", "High") => (4.6, 0.3),  // ~$100 median
        // ... more combinations
        _ => (3.5, 0.5),  // Default
    };
    
    // Log-normal distribution for most expense categories
    let log_normal = LogNormal::new(mu, sigma).unwrap();
    log_normal.sample(rng).round() * 0.01 * 100.0  // Round to nearest cent
}
```

### 2. Recurring Bills

```rust
// For recurring bills, generate consistent amounts with small variations
fn generate_recurring_bills(customer: &Customer, rng: &mut StdRng) -> Vec<RecurringBill> {
    let mut bills = Vec::new();
    
    // Rent/Mortgage (scaled to income)
    let housing_ratio = match customer.income_bracket {
        "High" => uniform_distribution.sample(rng) * 0.1 + 0.2,    // 20-30% of income
        "Medium" => uniform_distribution.sample(rng) * 0.1 + 0.25, // 25-35% of income
        "Low" => uniform_distribution.sample(rng) * 0.1 + 0.3,     // 30-40% of income
    };
    
    let housing_amount = customer.base_income * housing_ratio;
    let housing_day = uniform_int_distribution(1, 5).sample(rng);
    
    bills.push(RecurringBill {
        description: "Rent/Mortgage Payment".to_string(),
        category: "Housing".to_string(),
        amount: housing_amount,
        day: housing_day,
        variance: 0.0,  // Fixed amount
    });
    
    // Utilities (with seasonal variance)
    let utility_amount = customer.base_income * 
        (uniform_distribution.sample(rng) * 0.03 + 0.02);  // 2-5% of income
        
    bills.push(RecurringBill {
        description: "Utility Bill".to_string(),
        category: "Utilities".to_string(),
        amount: utility_amount,
        day: uniform_int_distribution(10, 20).sample(rng),
        variance: 0.3,  // High variance (seasonal)
    });
    
    // Subscriptions (multiple small ones)
    let subscription_count = poisson_distribution(3.0).sample(rng);
    for i in 0..subscription_count {
        let amount = 5.0 + geometric_distribution(0.7).sample(rng) as f64 * 5.0;  // $5-30
        
        bills.push(RecurringBill {
            description: format!("Subscription {}", i+1),
            category: "Subscription".to_string(),
            amount,
            day: uniform_int_distribution(1, 28).sample(rng),
            variance: 0.0,  // Fixed amount
        });
    }
    
    bills
}
```

## Implementation Challenges and Solutions

### 1. Balancing Determinism with Variability

To ensure the data is both deterministic (reproducible) but still variable enough to feel realistic:

```rust
// Use a fixed seed for the overall generation process
let base_seed = 42;

// But derive customer-specific seeds to get variability between customers
for customer_id in 1..=customer_count {
    let customer_seed = base_seed ^ customer_id;
    let mut customer_rng = StdRng::seed_from_u64(customer_seed);
    
    // Generate customer-specific data
    generate_customer_transactions(customer_id, &mut customer_rng);
}
```

### 2. Handling Special Cases (Holidays, Weekends)

```rust
// Define calendar of special days
let holidays = [
    (1, 1),    // New Year's
    (12, 25),  // Christmas
    // more holidays...
];

// Check if date is a holiday or weekend
fn is_special_day(date: NaiveDate) -> bool {
    let day_of_week = date.weekday().num_days_from_sunday();
    let is_weekend = day_of_week == 0 || day_of_week == 6;
    
    let is_holiday = holidays.contains(&(date.month(), date.day()));
    
    is_weekend || is_holiday
}

// Adjust payment dates for holidays/weekends
fn adjust_payment_date(date: NaiveDate) -> NaiveDate {
    if is_special_day(date) {
        // Move to next business day
        (1..=5).find_map(|days| {
            let new_date = date + Duration::days(days);
            if !is_special_day(new_date) {
                Some(new_date)
            } else {
                None
            }
        }).unwrap_or(date + Duration::days(1))
    } else {
        date
    }
}
```

## Time-Series Generation Example

Here's a concrete example of generating one month of transactions for a medium-income customer:

```rust
let customer = Customer {
    id: 42,
    name: "Jane Smith",
    income_bracket: "Medium",
    base_income: 5000.0,
    pay_frequency: "monthly",
    // other fields...
};

let account = Account {
    id: 101,
    customer_id: 42,
    initial_balance: 2500.0,
    overdraft_limit: 1000.0,
    // other fields...
};

// Month to generate (May 2024)
let year = 2024;
let month = 5;

// Generate transactions for this month
let mut transactions = Vec::new();

// 1. Income deposit (around 1st of month)
let income_date = get_date(year, month, 1 + rng.gen_range(0..=2));
transactions.push(Transaction {
    account_id: account.id,
    datetime: combine_date_time(income_date, morning_time(rng)),
    amount: 5000.0 * (1.0 + normal_distribution.sample(rng) * 0.02),
    category: "Income",
    description: "Salary Deposit",
    is_recurring: true,
    // other fields...
});

// 2. Recurring bills
let bills = [
    ("Rent", 1600.0, 2),
    ("Electric Bill", 120.0, 15),
    ("Internet", 80.0, 10),
    ("Phone Bill", 95.0, 12),
    ("Streaming Service", 15.0, 5),
    ("Gym Membership", 50.0, 8),
];

for (description, amount, day) in bills {
    let bill_date = get_date(year, month, day);
    transactions.push(Transaction {
        account_id: account.id,
        datetime: combine_date_time(bill_date, business_time(rng)),
        amount: -amount * (1.0 + normal_distribution.sample(rng) * 0.05),
        category: "Bill",
        description,
        is_recurring: true,
        // other fields...
    });
}

// 3. Regular daily expenses
// 3a. Weekday lunch (work days)
for day in 1..=31 {
    let date = match get_date_opt(year, month, day) {
        Some(d) => d,
        None => break,  // End of month
    };
    
    let day_of_week = date.weekday().num_days_from_sunday();
    
    // Skip weekends for work lunch
    if day_of_week > 0 && day_of_week < 6 {
        // 70% chance of buying lunch on workdays
        if rng.gen::<f64>() < 0.7 {
            transactions.push(Transaction {
                account_id: account.id,
                datetime: combine_date_time(date, lunch_time(rng)),
                amount: -lognormal_amount(2.5, 0.3, rng),  // ~$12 lunch
                category: "Dining",
                description: "Lunch",
                is_recurring: false,
                // other fields...
            });
        }
    }
    
    // Weekend entertainment (higher probability on Fri/Sat)
    let entertainment_prob = match day_of_week {
        5 => 0.8,  // Friday
        6 => 0.7,  // Saturday
        0 => 0.3,  // Sunday
        _ => 0.2,  // Weekday
    };
    
    if rng.gen::<f64>() < entertainment_prob {
        transactions.push(Transaction {
            account_id: account.id,
            datetime: combine_date_time(date, evening_time(rng)),
            amount: -lognormal_amount(3.4, 0.5, rng),  // ~$30 entertainment
            category: "Entertainment",
            description: "Evening Out",
            is_recurring: false,
            // other fields...
        });
    }
    
    // Groceries (higher probability on Sunday, lower mid-week)
    let grocery_prob = match day_of_week {
        0 => 0.7,  // Sunday
        3 => 0.4,  // Wednesday
        _ => 0.1,  // Other days
    };
    
    if rng.gen::<f64>() < grocery_prob {
        transactions.push(Transaction {
            account_id: account.id,
            datetime: combine_date_time(date, evening_time(rng)),
            amount: -lognormal_amount(4.0, 0.4, rng),  // ~$55 groceries
            category: "Groceries",
            description: "Grocery Shopping",
            is_recurring: false,
            // other fields...
        });
    }
}

// Sort chronologically
transactions.sort_by(|a, b| a.datetime.cmp(&b.datetime));

// This yields a realistic month of transactions with:
// - Regular income
// - Fixed bills
// - Day-of-week patterns for different expense types
// - Realistic timing and amounts
```

## Conclusion

This data generation strategy creates a rich, realistic financial dataset with these time-series characteristics:

1. **Multi-level temporal patterns**:
   - Daily patterns (time-of-day appropriate transactions)
   - Weekly patterns (weekday vs weekend spending)
   - Monthly patterns (income and bill cycles)
   - Seasonal patterns (weather-affected utilities, holiday spending)
   - Annual patterns (yearly events and celebrations)

2. **Realistic financial behaviors**:
   - Income-driven spending
   - Recurring bills and subscriptions
   - Variable daily expenses
   - Occasional large purchases
   - Balance maintenance and cash flow management

3. **Statistical realism**:
   - Appropriate distributions for different transaction types
   - Coherent relationships between transactions
   - Realistic merchant affinities
   - Income-appropriate spending amounts

By implementing this strategy, the FeatherFlow Financial Demo will provide a compelling dataset that enables meaningful time-series analysis while maintaining deterministic reproducibility.