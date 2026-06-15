# LotusX 产品需求文档 (PRD)
## 多交易所加密货币永续合约交易平台

### 执行摘要

LotusX是一个基于Rust的库，旨在通过标准化API接口提供对多个加密货币交易所的统一访问。该项目专注于永续合约市场，通过为与各种交易所交互提供单一、一致的接口来解决加密货币衍生品交易中的碎片化问题。

---

## 1. 问题陈述与市场需求

### 当前痛点：
- **API碎片化**：每个交易所都有不同的API结构、认证方法和数据格式
- **永续合约复杂性**：衍生品交易需要更复杂的风险管理和持仓跟踪
- **REST vs WebSocket差异**：交易所在REST API和WebSocket实现上存在显著差异
  - **REST API**：适合订单操作、账户查询等请求-响应场景
  - **WebSocket**：适合实时市场数据、价格流、持仓更新等流式数据
- **开发开销**：交易者和开发者需要学习和实现多个交易所API
- **风险管理**：缺乏跨交易所的统一持仓和风险管理
- **实时数据**：各交易所WebSocket实现不一致，延迟差异大

### 目标用户：
- **永续合约交易者**：需要程序化访问多个衍生品交易平台
- **套利交易者**：需要同时监控多个交易所的永续合约价格差异
- **做市商**：需要高频交易和实时风险管理
- **投资组合管理者**：需要跨交易所统一持仓视图
- **量化基金**：构建衍生品交易策略和风险管理系统

---

## 2. 产品愿景与目标

### 愿景声明：
"成为永续合约交易的统一接口标准，为开发者提供高性能、低延迟的多交易所衍生品交易解决方案。"

### 关键目标：
1. **永续合约专业化**：专注于衍生品市场的特殊需求
2. **REST + WebSocket统一**：提供协调的REST和WebSocket接口
3. **低延迟**：针对高频衍生品交易优化
4. **风险管理**：内置持仓跟踪和风险计算
5. **资金费率监控**：实时资金费率和基差跟踪

---

## 3. 核心需求

### 3.1 功能需求

#### 第一阶段 (MVP) - 核心永续合约交易
**优先级：P0（必须具备）**

1. **交易所连接**
   - Binance永续期货 ✅ （已实现）
   - Hyperliquid（下一优先级）
   - Bybit（下一优先级）

2. **永续合约交易操作**
   - 开仓/平仓订单 ✅ （已实现）
   - 杠杆调整
   - 持仓管理
   - 保证金管理
   - 强制平仓监控

3. **市场数据（双接口架构）**

   **REST API接口**：
   - 获取合约信息
   - 历史K线数据
   - 账户余额查询
   - 持仓信息查询
   - 订单历史

   **WebSocket流接口**：
   - 实时价格流 ✅ （已实现）
   - 订单簿深度流 ✅ （已实现）
   - 交易执行流 ✅ （已实现）
   - 持仓变化推送
   - 保证金变化推送
   - 资金费率实时更新

4. **永续合约特有功能**
   - 资金费率监控
   - 标记价格跟踪
   - 指数价格获取
   - 基差计算
   - 强制平仓价格计算

#### 第二阶段 - 高级衍生品功能
**优先级：P1（应该具备）**

1. **跨交易所套利**
   - 价差监控
   - 套利机会识别
   - 风险中性持仓管理

2. **高级订单类型**
   - 条件订单
   - 冰山订单
   - 时间加权订单

---

## 4. 首次发布MVP功能

### 4.1 核心交易所支持（专注永续合约）
**V1.0必须具备的交易所：**

1. **Binance永续** ✅ （已实现）
   - USDT保证金合约
   - COIN保证金合约
   - 资金费率API

2. **Hyperliquid**（新增）
   - 原生永续合约
   - 高性能订单簿
   - 链上结算

3. **Bybit**（新增）
   - USDT永续合约
   - 反向永续合约
   - 期权合约

### 4.2 双接口架构设计

```rust
// 永续合约专用接口
pub trait PerpetualExchangeConnector {
    // REST API - 请求响应模式
    async fn get_perpetual_contracts(&self) -> Result<Vec<PerpContract>>;
    async fn get_funding_rate(&self, symbol: &str) -> Result<FundingRate>;
    async fn get_position(&self, symbol: &str) -> Result<Position>;
    async fn place_perpetual_order(&self, order: PerpOrderRequest) -> Result<OrderResponse>;
    async fn adjust_leverage(&self, symbol: &str, leverage: u8) -> Result<LeverageResponse>;

    // WebSocket - 流式数据模式
    async fn subscribe_price_stream(&self, symbols: Vec<String>) -> Result<PriceStream>;
    async fn subscribe_position_updates(&self) -> Result<PositionStream>;
    async fn subscribe_funding_rate_updates(&self) -> Result<FundingStream>;
    async fn subscribe_liquidation_stream(&self) -> Result<LiquidationStream>;
}
```

### 4.3 WebSocket vs REST API差异处理

1. **REST API用途**
   - 一次性数据查询
   - 订单操作
   - 账户状态查询
   - 历史数据获取
   - 配置变更

2. **WebSocket用途**
   - 实时价格推送
   - 订单簿变化
   - 交易执行通知
   - 持仓实时更新
   - 保证金告警

3. **统一数据模型**
   - REST和WebSocket返回相同的数据结构
   - 自动数据同步和一致性保证
   - 智能缓存和去重机制

---

## 5. 技术架构

### 5.1 永续合约专用架构

```
TBD
```

### 5.2 双接口协调机制

1. **连接管理**
   - REST连接池复用
   - WebSocket长连接维护
   - 自动故障切换

2. **数据一致性**
   - WebSocket数据优先
   - REST数据作为备份和验证
   - 增量更新机制

---

## 6. 永续合约特有指标

### 6.1 交易指标
- **资金费率**：实时监控和历史分析
- **基差**：现货-期货价差跟踪
- **持仓量**：未平仓合约统计
- **强制平仓**：清算事件监控

### 6.2 风险指标
- **保证金比率**：实时风险监控
- **杠杆使用率**：跨交易所杠杆统计
- **PnL追踪**：已实现/未实现盈亏

---

## 7. 实施路线图

### 第一阶段（第1-3个月）：永续合约基础
- ✅ Binance永续期货集成
- Hyperliquid集成
- 资金费率监控系统
- 基础持仓管理

### 第二阶段（第4-6个月）：多交易所
- Bybit永续合约集成
- 跨交易所套利框架
- 高级风险管理
- 实时监控仪表板

### 第三阶段（第7-12个月）：策略引擎
- 做市策略框架
- 套利策略模板
- 回测引擎
- 性能分析工具

---

## 8. 结论

LotusX专注于永续合约市场，通过统一的REST API和WebSocket接口，为衍生品交易者提供高性能、低延迟的多交易所访问解决方案。

MVP将专注于Binance、Hyperliquid、Bybit三大永续合约平台，提供完整的持仓管理、资金费率监控和实时风险管理功能，为专业衍生品交易者和量化基金提供强大的技术基础。