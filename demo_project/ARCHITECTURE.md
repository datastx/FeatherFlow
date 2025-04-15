# FeatherFlow Financial Demo Architecture

This document provides an architectural overview of the FeatherFlow Financial Demo, focusing on data flow, transformation layers, and time-series analysis capabilities.

## System Architecture

The FeatherFlow Financial Demo consists of several connected components that work together to generate, store, transform, and analyze financial data.

```mermaid
graph TD
    subgraph "Data Generation"
        A[Rust Data Generator] --> B[Synthetic Financial Data]
        B --> C[CSV Files]
    end
    
    subgraph "Storage"
        C --> D[DuckDB Database]
    end
    
    subgraph "Transformation Layers"
        D --> E[Raw Data Schema]
        E --> F[Staging Models]
        F --> G[Core Mart Models]
        F --> H[Finance Mart Models]
    end
    
    subgraph "Analysis & Visualization"
        G --> I[Business Metrics]
        H --> J[Time-Series Analysis]
        I --> K[Visualizations]
        J --> K
    end
```

## Data Flow

The data flows through the system as follows:

1. **Generation**: The Rust-based data generator creates synthetic financial data with time-series patterns
2. **Storage**: Data is written to CSV files and loaded into a DuckDB database
3. **Staging**: Raw data is cleaned and standardized in the staging layer
4. **Transformation**: Staging data is transformed into business-level metrics and time-series analytics
5. **Analysis**: Transformed data is used for business insights and visualization

## Data Model

The financial demo uses a star schema centered around transaction data:

```mermaid
erDiagram
    CUSTOMERS ||--o{ ACCOUNTS : has
    ACCOUNTS ||--o{ TRANSACTIONS : contains
    MERCHANTS ||--o{ TRANSACTIONS : processes
    CREDIT_CARDS ||--o{ TRANSACTIONS : generates
    CUSTOMERS ||--o{ CREDIT_CARDS : owns
    CUSTOMERS ||--o{ LOANS : takes
    
    CUSTOMERS {
        int customer_id PK
        string name
        string email
        string address
        date registration_date
        int credit_score
        string income_bracket
    }
    
    ACCOUNTS {
        int account_id PK
        int customer_id FK
        string account_type
        date open_date
        string status
        string currency
        decimal initial_balance
        decimal current_balance
    }
    
    TRANSACTIONS {
        int transaction_id PK
        int account_id FK
        int merchant_id FK
        int card_id FK
        timestamp transaction_datetime
        decimal amount
        string transaction_type
        string description
        string category
        string status
        bool is_recurring
        int day_of_week
        int month
        int year
        string time_of_day
    }
    
    MERCHANTS {
        int merchant_id PK
        string name
        string category
        string location
        bool is_online
        float popularity_score
    }
    
    CREDIT_CARDS {
        int card_id PK
        int customer_id FK
        string card_type
        decimal credit_limit
        date issue_date
        date expiry_date
        string status
    }
    
    LOANS {
        int loan_id PK
        int customer_id FK
        string loan_type
        decimal principal_amount
        decimal interest_rate
        int term_months
        date start_date
        string status
    }
```

## Transformation Architecture

The transformation process follows a layered approach:

```mermaid
flowchart TD
    subgraph "Raw Data"
        A1[customers.csv]
        A2[accounts.csv]
        A3[transactions.csv]
        A4[merchants.csv]
        A5[credit_cards.csv]
        A6[loans.csv]
    end
    
    subgraph "Staging Models"
        B1[stg_customers.sql]
        B2[stg_accounts.sql]
        B3[stg_transactions.sql]
        B4[stg_merchants.sql]
        B5[stg_credit_cards.sql]
        B6[stg_loans.sql]
    end
    
    subgraph "Core Mart Models"
        C1[customer_summary.sql]
        C2[merchant_summary.sql]
    end
    
    subgraph "Finance Mart Models"
        D1[daily_trends.sql]
        D2[monthly_trends.sql]
        D3[spending_categories.sql]
        D4[recurring_analysis.sql]
        D5[seasonal_patterns.sql]
    end
    
    A1 --> B1
    A2 --> B2
    A3 --> B3
    A4 --> B4
    A5 --> B5
    A6 --> B6
    
    B1 --> C1
    B2 --> C1
    B3 --> C1
    B5 --> C1
    B6 --> C1
    
    B3 --> C2
    B4 --> C2
    
    B3 --> D1
    B4 --> D1
    
    B3 --> D2
    B4 --> D2
    
    B3 --> D3
    B4 --> D3
    
    B1 --> D4
    B2 --> D4
    B3 --> D4
    B4 --> D4
    
    B3 --> D5
    B4 --> D5
```

## Time-Series Analysis Architecture

The time-series analysis capabilities include:

```mermaid
graph TD
    A[Transaction Data] --> B[Temporal Decomposition]
    B --> C[Day of Week Component]
    B --> D[Month Component] 
    B --> E[Year Component]
    B --> F[Time of Day Component]
    
    A --> G[Aggregation Models]
    G --> H[Daily Trends]
    G --> I[Monthly Trends]
    G --> J[Seasonal Patterns]
    
    A --> K[Pattern Recognition]
    K --> L[Recurring Transactions]
    K --> M[Spending Categories]
    
    A --> N[Time Series Forecasting]
    N --> O[Moving Averages]
    N --> P[Year-over-Year Comparison]
    N --> Q[Seasonal Adjustments]
```

## CLI Command Structure

The FeatherFlow CLI is extended with the demo subcommand:

```mermaid
graph TD
    A[ff CLI] --> B[parse]
    A --> C[version]
    A --> D[demo]
    
    D --> E[init]
    D --> F[generate]
    D --> G[load]
    D --> H[transform]
    D --> I[visualize]
    
    F --> J[customers parameter]
    F --> K[transactions parameter]
    F --> L[days parameter]
    
    G --> M[db_path parameter]
    
    H --> N[db_path parameter]
    H --> O[target parameter]
    
    I --> P[db_path parameter]
    I --> Q[output_dir parameter]
```

## Implementation Components

The implementation consists of these main components:

```mermaid
classDiagram
    class DemoCommand {
        +init_command()
        +generate_command()
        +load_command()
        +transform_command()
        +visualize_command()
    }
    
    class DataGenerator {
        +generate_customers()
        +generate_accounts()
        +generate_transactions()
        +generate_merchants()
        +generate_credit_cards()
        +generate_loans()
    }
    
    class TimeSeriesPatterns {
        +GenerateRecurringTransactions()
        +GenerateSeasonalPatterns()
        +GenerateDailyPatterns()
        +GenerateHourlyPatterns()
    }
    
    class DataLoader {
        +write_to_csv()
        +create_and_load_table()
    }
    
    class Transformer {
        +run_sql_model()
        +execute_staging_models()
        +execute_mart_models()
    }
    
    DemoCommand --> DataGenerator
    DemoCommand --> DataLoader
    DemoCommand --> Transformer
    DataGenerator --> TimeSeriesPatterns
```

## Time-Series Features

The time-series features of the financial demo include:

### 1. Temporal Decomposition

Each transaction is decomposed into temporal components:
- Year
- Month
- Day of month
- Day of week
- Time of day
- Season

### 2. Recurring Pattern Generation

The data generator simulates realistic recurring patterns:
- Biweekly or monthly income deposits
- Monthly bill payments on consistent dates
- Weekly recurring expenses (e.g., groceries)
- Subscriptions with fixed intervals

### 3. Seasonal Patterns

Spending patterns vary by season:
- Holiday shopping in winter
- Travel expenses in summer
- Back-to-school spending in fall

### 4. Year-over-Year Analysis

Models include comparison of metrics across years:
- Spending growth rates
- Category shifts
- Merchant popularity changes

### 5. Moving Averages and Trends

Analysis includes trend detection:
- 3-month rolling averages
- 6-month rolling averages
- Month-over-month growth rates
- Linear trend projections

### 6. Time-Based Forecasting

Basic forecasting techniques:
- Weighted moving averages
- Seasonal adjustments
- Year-over-year extrapolation

## Conclusion

The FeatherFlow Financial Demo architecture provides a comprehensive framework for generating, transforming, and analyzing time-series financial data. The layered approach to transformations allows for clear separation of concerns, while the integrated time-series components enable sophisticated financial analysis.

This architecture can be extended with additional features such as:
1. Anomaly detection for fraud identification
2. Advanced forecasting models
3. Interactive dashboards
4. Portfolio optimization
5. Risk modeling