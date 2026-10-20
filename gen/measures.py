"""Measure templates (source of data/measures.json).  M(id, name, category, description, body, params, assumptions)
params: "name:type:description:example"  (type: table | column | measure | expression | name | number | text)
body uses {{param}} placeholders; the renderer prefixes it with the `measure_name` parameter ("<name> = <body>")."""

MEASURES = []


def M(id_, name, cat, desc, body, params, assume=""):
    ps = [{"name": "measure_name", "type": "name", "description": "Name of the measure to create", "example": name}]
    for p in params:
        n, t, d, e = p.split(":", 3)
        ps.append({"name": n, "type": t, "description": d, "example": e})
    MEASURES.append({"id": id_, "name": name, "category": cat, "description": desc, "body": body.strip(), "parameters": ps,
                     "assumptions": assume})


SALES = "sales:column:Numeric column to add up:'Sales'[Amount]"
DATES = "dates:column:Date column of a marked date table:'Date'[Date]"
DT = "Requires a date table related to the fact table and marked as the model's date table."
M("sum", "Total", "Basic", "Adds a numeric column.", "SUM({{column}})", ["column:column:Column to add up:'Sales'[Amount]"])
M("average", "Average", "Basic", "Mean of a numeric column.", "AVERAGE({{column}})", ["column:column:Column to average:'Sales'[Amount]"])
M("min", "Minimum", "Basic", "Smallest value of a column.", "MIN({{column}})", ["column:column:Column:'Sales'[Amount]"])
M("max", "Maximum", "Basic", "Largest value of a column.", "MAX({{column}})", ["column:column:Column:'Sales'[Amount]"])
M("median", "Median", "Basic", "Median of a numeric column.", "MEDIAN({{column}})", ["column:column:Column:'Sales'[Amount]"])
M("count-rows", "Row count", "Basic", "Counts the rows of a table in the current filter context.", "COUNTROWS({{table}})", ["table:table:Table to count:'Sales'"])
M("count-nonblank", "Non-blank count", "Basic", "Counts the non-blank values of a column.", "COUNTA({{column}})", ["column:column:Column:'Sales'[OrderID]"])
M("distinct-count", "Distinct count", "Basic", "Counts the different values of a column.", "DISTINCTCOUNT({{column}})", ["column:column:Column:'Sales'[CustomerID]"])
M("distinct-count-noblank", "Distinct count (no blank)", "Basic", "Counts the different values of a column, blank excluded.", "DISTINCTCOUNTNOBLANK({{column}})", ["column:column:Column:'Sales'[CustomerID]"])
M("sumx-product", "Sum of price x quantity", "Basic", "Row-by-row product of two columns, then added.", "SUMX({{table}}, {{price}} * {{quantity}})",
  ["table:table:Fact table:'Sales'", "price:column:Unit price column:'Sales'[UnitPrice]", "quantity:column:Quantity column:'Sales'[Quantity]"])
M("weighted-average", "Weighted average", "Basic", "Average of a value weighted by another column.", "DIVIDE(SUMX({{table}}, {{value}} * {{weight}}), SUM({{weight}}))",
  ["table:table:Table:'Sales'", "value:column:Value column:'Sales'[Price]", "weight:column:Weight column:'Sales'[Quantity]"])
M("ratio", "Ratio", "Basic", "Safe division of two measures (blank when the denominator is 0).", "DIVIDE({{numerator}}, {{denominator}})",
  ["numerator:measure:Numerator:[Profit]", "denominator:measure:Denominator:[Sales]"])
M("margin-pct", "Margin %", "Basic", "Profit as a share of revenue.", "DIVIDE({{revenue}} - {{cost}}, {{revenue}})",
  ["revenue:measure:Revenue measure:[Sales]", "cost:measure:Cost measure:[Cost]"])
M("average-per-distinct", "Average per distinct", "Basic", "A measure divided by the distinct count of a column (e.g. sales per customer).", "DIVIDE({{measure}}, DISTINCTCOUNT({{column}}))",
  ["measure:measure:Measure:[Sales]", "column:column:Column to count:'Sales'[CustomerID]"])

M("ytd", "YTD", "Time intelligence", "Year-to-date total of a measure.", "TOTALYTD({{measure}}, {{dates}})", ["measure:measure:Measure:[Sales]", DATES], DT)
M("ytd-fiscal", "Fiscal YTD", "Time intelligence", "Year-to-date with a fiscal year end (text like \"6/30\").", "TOTALYTD({{measure}}, {{dates}}, {{year_end}})",
  ["measure:measure:Measure:[Sales]", DATES, "year_end:text:Fiscal year end date:6/30"], DT)
M("qtd", "QTD", "Time intelligence", "Quarter-to-date total of a measure.", "TOTALQTD({{measure}}, {{dates}})", ["measure:measure:Measure:[Sales]", DATES], DT)
M("mtd", "MTD", "Time intelligence", "Month-to-date total of a measure.", "TOTALMTD({{measure}}, {{dates}})", ["measure:measure:Measure:[Sales]", DATES], DT)
M("prior-year", "Prior year", "Time intelligence", "The measure for the same period one year earlier.", "CALCULATE({{measure}}, SAMEPERIODLASTYEAR({{dates}}))", ["measure:measure:Measure:[Sales]", DATES], DT)
M("yoy-abs", "YoY change", "Time intelligence", "Difference against the same period last year.", "{{measure}} - CALCULATE({{measure}}, SAMEPERIODLASTYEAR({{dates}}))", ["measure:measure:Measure:[Sales]", DATES], DT)
M("yoy-pct", "YoY %", "Time intelligence", "Growth against the same period last year.",
  "VAR _prior = CALCULATE({{measure}}, SAMEPERIODLASTYEAR({{dates}}))\nRETURN DIVIDE({{measure}} - _prior, _prior)", ["measure:measure:Measure:[Sales]", DATES], DT)
M("prior-month", "Prior month", "Time intelligence", "The measure for the previous month.", "CALCULATE({{measure}}, DATEADD({{dates}}, -1, MONTH))", ["measure:measure:Measure:[Sales]", DATES], DT)
M("mom-pct", "MoM %", "Time intelligence", "Growth against the previous month.",
  "VAR _prior = CALCULATE({{measure}}, DATEADD({{dates}}, -1, MONTH))\nRETURN DIVIDE({{measure}} - _prior, _prior)", ["measure:measure:Measure:[Sales]", DATES], DT)
M("prior-quarter", "Prior quarter", "Time intelligence", "The measure for the previous quarter.", "CALCULATE({{measure}}, DATEADD({{dates}}, -1, QUARTER))", ["measure:measure:Measure:[Sales]", DATES], DT)
M("qoq-pct", "QoQ %", "Time intelligence", "Growth against the previous quarter.",
  "VAR _prior = CALCULATE({{measure}}, DATEADD({{dates}}, -1, QUARTER))\nRETURN DIVIDE({{measure}} - _prior, _prior)", ["measure:measure:Measure:[Sales]", DATES], DT)
M("ytd-vs-prior-ytd", "YTD vs prior YTD %", "Time intelligence", "Year-to-date growth against the previous year-to-date.",
  "VAR _ytd = TOTALYTD({{measure}}, {{dates}})\nVAR _pytd = CALCULATE(TOTALYTD({{measure}}, {{dates}}), SAMEPERIODLASTYEAR({{dates}}))\nRETURN DIVIDE(_ytd - _pytd, _pytd)",
  ["measure:measure:Measure:[Sales]", DATES], DT)
M("rolling-12m", "Rolling 12 months", "Time intelligence", "Total of the last 12 months ending at the last date in context.",
  "CALCULATE({{measure}}, DATESINPERIOD({{dates}}, MAX({{dates}}), -12, MONTH))", ["measure:measure:Measure:[Sales]", DATES], DT)
M("rolling-days", "Rolling N days", "Time intelligence", "Total of the last N days ending at the last date in context.",
  "CALCULATE({{measure}}, DATESINPERIOD({{dates}}, MAX({{dates}}), -{{days}}, DAY))", ["measure:measure:Measure:[Sales]", DATES, "days:number:Number of days:30"], DT)
M("moving-average-months", "Moving average (months)", "Time intelligence", "Average monthly value over the last N months.",
  "AVERAGEX(DATESINPERIOD({{dates}}, MAX({{dates}}), -{{months}}, MONTH), {{measure}})", ["measure:measure:Measure:[Sales]", DATES, "months:number:Window in months:3"], DT)
M("running-total", "Running total", "Time intelligence", "Cumulative total from the first date up to the last date in context.",
  "CALCULATE({{measure}}, FILTER(ALL({{date_table}}), {{dates}} <= MAX({{dates}})))", ["measure:measure:Measure:[Sales]", DATES, "date_table:table:Date table:'Date'"], DT)
M("opening-balance-month", "Opening balance (month)", "Time intelligence", "Balance at the day before the month starts.", "OPENINGBALANCEMONTH({{measure}}, {{dates}})", ["measure:measure:Balance measure:[Balance]", DATES], DT)
M("closing-balance-month", "Closing balance (month)", "Time intelligence", "Balance at the last date of the month.", "CLOSINGBALANCEMONTH({{measure}}, {{dates}})", ["measure:measure:Balance measure:[Balance]", DATES], DT)
M("closing-balance-year", "Closing balance (year)", "Time intelligence", "Balance at the last date of the year.", "CLOSINGBALANCEYEAR({{measure}}, {{dates}})", ["measure:measure:Balance measure:[Balance]", DATES], DT)
M("last-date-with-data", "Last date with data", "Time intelligence", "Latest date on which a fact column has a value.", "CALCULATE(MAX({{dates}}), FILTER(ALL({{date_table}}), CALCULATE(COUNTROWS({{fact}})) > 0))",
  [DATES, "date_table:table:Date table:'Date'", "fact:table:Fact table:'Sales'"], DT)
M("days-since-last", "Days since last transaction", "Time intelligence", "Days between today and the last date with a transaction.", "DATEDIFF(MAX({{fact_dates}}), TODAY(), DAY)",
  ["fact_dates:column:Date column of the fact table:'Sales'[OrderDate]"])
M("last-n-days", "Last N days", "Time intelligence", "Total of the N days before today.", "CALCULATE({{measure}}, DATESINPERIOD({{dates}}, TODAY(), -{{days}}, DAY))", ["measure:measure:Measure:[Sales]", DATES, "days:number:Number of days:7"], DT)

M("pct-of-total", "% of total", "Share", "Share of the grand total ignoring all filters on a table.", "DIVIDE({{measure}}, CALCULATE({{measure}}, ALL({{table}})))", ["measure:measure:Measure:[Sales]", "table:table:Table whose filters are removed:'Product'"])
M("pct-of-visible-total", "% of visible total", "Share", "Share of the total of the rows shown in the visual.", "DIVIDE({{measure}}, CALCULATE({{measure}}, ALLSELECTED({{table}})))", ["measure:measure:Measure:[Sales]", "table:table:Table:'Product'"])
M("pct-of-column-total", "% of column total", "Share", "Share of the total when one column's filter is removed.", "DIVIDE({{measure}}, CALCULATE({{measure}}, REMOVEFILTERS({{column}})))", ["measure:measure:Measure:[Sales]", "column:column:Column:'Product'[Category]"])
M("pct-of-parent", "% of parent", "Share", "Share of the parent group (keeps the filter of the parent column).", "DIVIDE({{measure}}, CALCULATE({{measure}}, ALLEXCEPT({{table}}, {{parent}})))", ["measure:measure:Measure:[Sales]", "table:table:Table:'Product'", "parent:column:Parent column kept in the filter:'Product'[Category]"])
M("share-of-selected", "Share of selection", "Share", "Share of the sum of the currently selected values of a column.", "DIVIDE({{measure}}, CALCULATE({{measure}}, ALLSELECTED({{column}})))", ["measure:measure:Measure:[Sales]", "column:column:Column:'Product'[Name]"])
M("pareto-cumulative", "Cumulative % (Pareto)", "Share", "Cumulative share of items ranked from largest to smallest.",
  "VAR _v = {{measure}}\nVAR _items = ALLSELECTED({{column}})\nVAR _cum = SUMX(FILTER(_items, {{measure}} >= _v), {{measure}})\nRETURN DIVIDE(_cum, SUMX(_items, {{measure}}))", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]"])

M("rank-all", "Rank", "Ranking", "Rank of the current item among all items (largest = 1).", "RANKX(ALL({{column}}), {{measure}}, , DESC, Dense)", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]"])
M("rank-selected", "Rank in selection", "Ranking", "Rank among the items shown in the visual.", "RANKX(ALLSELECTED({{column}}), {{measure}}, , DESC, Dense)", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]"])
M("rank-in-group", "Rank within group", "Ranking", "Rank of an item inside its group (e.g. product inside category).", "RANKX(ALLEXCEPT({{table}}, {{group}}), {{measure}}, , DESC, Dense)", ["measure:measure:Measure:[Sales]", "table:table:Table:'Product'", "group:column:Group column:'Product'[Category]"])
M("top-n-sum", "Top N total", "Ranking", "Total of the N best items.", "SUMX(TOPN({{n}}, ALLSELECTED({{column}}), {{measure}}, DESC), {{measure}})", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]", "n:number:How many items:5"])
M("top-n-share", "Top N share", "Ranking", "Share of the total held by the N best items.", "DIVIDE(SUMX(TOPN({{n}}, ALLSELECTED({{column}}), {{measure}}, DESC), {{measure}}), CALCULATE({{measure}}, ALLSELECTED({{column}})))", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]", "n:number:How many items:5"])
M("bottom-n-sum", "Bottom N total", "Ranking", "Total of the N worst items.", "SUMX(TOPN({{n}}, ALLSELECTED({{column}}), {{measure}}, ASC), {{measure}})", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]", "n:number:How many items:5"])
M("top-n-only", "Top N items only", "Ranking", "Shows the measure only for the N best items (blank for the rest); use in a visual.", "IF(RANKX(ALLSELECTED({{column}}), {{measure}}, , DESC, Dense) <= {{n}}, {{measure}})", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]", "n:number:How many items:10"])

M("filtered-value", "Filtered measure", "Filter", "A measure restricted to one value of a column.", "CALCULATE({{measure}}, {{column}} = {{value}})", ["measure:measure:Measure:[Sales]", "column:column:Column to filter:'Product'[Color]", "value:expression:Value (DAX literal):\"Red\""])
M("ignore-filter", "Measure ignoring a filter", "Filter", "A measure that ignores the filters of one column.", "CALCULATE({{measure}}, REMOVEFILTERS({{column}}))", ["measure:measure:Measure:[Sales]", "column:column:Column:'Product'[Category]"])
M("keep-filters", "Measure with extra filter kept", "Filter", "Adds a condition without overriding the visual's own filter on the column.", "CALCULATE({{measure}}, KEEPFILTERS({{column}} = {{value}}))", ["measure:measure:Measure:[Sales]", "column:column:Column:'Product'[Color]", "value:expression:Value (DAX literal):\"Red\""])
M("inactive-relationship", "Measure via inactive relationship", "Filter", "Uses an inactive relationship (e.g. ship date instead of order date).", "CALCULATE({{measure}}, USERELATIONSHIP({{fact_column}}, {{dim_column}}))", ["measure:measure:Measure:[Sales]", "fact_column:column:Fact column:'Sales'[ShipDate]", "dim_column:column:Dimension column:'Date'[Date]"])
M("count-with-condition", "Count rows with condition", "Filter", "Counts rows of a table that meet a condition.", "CALCULATE(COUNTROWS({{table}}), {{condition}})", ["table:table:Table:'Sales'", "condition:expression:Boolean filter:'Sales'[Amount] > 1000"])
M("customers-with-sales", "Customers with sales", "Filter", "Distinct customers that have at least one sale in the context.", "CALCULATE(DISTINCTCOUNT({{customer}}), FILTER({{table}}, {{measure}} > 0))", ["customer:column:Customer key:'Sales'[CustomerID]", "table:table:Fact table:'Sales'", "measure:measure:Measure that must be > 0:[Sales]"])
M("new-customers", "New customers", "Filter", "Customers whose first purchase falls in the current period.", "COUNTROWS(FILTER(VALUES({{customer}}), CALCULATE(MIN({{order_date}}), ALL({{dates_table}})) >= MIN({{dates}}) && CALCULATE(MIN({{order_date}}), ALL({{dates_table}})) <= MAX({{dates}})))",
  ["customer:column:Customer key:'Sales'[CustomerID]", "order_date:column:Order date column:'Sales'[OrderDate]", "dates_table:table:Date table:'Date'", DATES], DT)
M("selected-value", "Selected value", "Filter", "The single selected value of a column, or a fallback text.", "SELECTEDVALUE({{column}}, {{fallback}})", ["column:column:Column:'Product'[Category]", "fallback:expression:Fallback (DAX literal):\"All\""])

M("stdev", "Standard deviation", "Statistics", "Sample standard deviation of a column.", "STDEV.S({{column}})", ["column:column:Column:'Sales'[Amount]"])
M("percentile", "Percentile", "Statistics", "Inclusive percentile of a column.", "PERCENTILE.INC({{column}}, {{k}})", ["column:column:Column:'Sales'[Amount]", "k:number:Percentile between 0 and 1:0.9"])
M("coefficient-variation", "Coefficient of variation", "Statistics", "Standard deviation divided by the mean.", "DIVIDE(STDEV.S({{column}}), AVERAGE({{column}}))", ["column:column:Column:'Sales'[Amount]"])
M("z-score", "Z-score", "Statistics", "Standard score of a measure across the items of a column.", "VAR _items = ALLSELECTED({{column}})\nRETURN DIVIDE({{measure}} - AVERAGEX(_items, {{measure}}), STDEVX.S(_items, {{measure}}))", ["measure:measure:Measure:[Sales]", "column:column:Item column:'Product'[Name]"])

M("title-text", "Dynamic title", "Formatting", "Title that includes the measure value and the selected category.", "{{label}} & \": \" & FORMAT({{measure}}, {{format}}) & \" | \" & SELECTEDVALUE({{column}}, \"All\")", ["label:text:Label:Sales", "measure:measure:Measure:[Sales]", "format:expression:Format string (DAX literal):\"#,0\"", "column:column:Column shown after the value:'Product'[Category]"])
M("kpi-status-color", "KPI status color", "Formatting", "Hex color for conditional formatting: green when the ratio reaches the target, red otherwise.", "IF(DIVIDE({{measure}}, {{target}}) >= 1, \"#1AAB40\", \"#D64554\")", ["measure:measure:Actual measure:[Sales]", "target:measure:Target measure:[Sales Target]"])
M("trend-arrow", "Trend arrow", "Formatting", "Text arrow showing whether the measure is above or below a comparison measure.", "IF({{measure}} > {{reference}}, \"▲\", IF({{measure}} < {{reference}}, \"▼\", \"●\"))", ["measure:measure:Measure:[Sales]", "reference:measure:Comparison measure:[Prior year]"])

CALC_ITEMS = []


def C(id_, name, desc, body, params, assume=DT):
    ps = []
    for p in params:
        n, t, d, e = p.split(":", 3)
        ps.append({"name": n, "type": t, "description": d, "example": e})
    ps.insert(0, {"name": "measure_name", "type": "name", "description": "Name of the calculation item", "example": name})
    MEASURES.append({"id": id_, "name": name, "category": "Calculation item", "description": desc, "body": body, "parameters": ps, "assumptions": assume})


C("calc-item-ytd", "YTD", "Calculation item: year-to-date of whatever measure is selected.", "CALCULATE(SELECTEDMEASURE(), DATESYTD({{dates}}))", [DATES])
C("calc-item-prior-year", "PY", "Calculation item: prior year of the selected measure.", "CALCULATE(SELECTEDMEASURE(), SAMEPERIODLASTYEAR({{dates}}))", [DATES])
C("calc-item-yoy-pct", "YoY %", "Calculation item: growth against last year for the selected measure.", "VAR _py = CALCULATE(SELECTEDMEASURE(), SAMEPERIODLASTYEAR({{dates}}))\nRETURN DIVIDE(SELECTEDMEASURE() - _py, _py)", [DATES])
C("calc-item-mom", "MoM", "Calculation item: change against the previous month for the selected measure.", "SELECTEDMEASURE() - CALCULATE(SELECTEDMEASURE(), DATEADD({{dates}}, -1, MONTH))", [DATES])
