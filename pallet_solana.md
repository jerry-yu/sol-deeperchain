deeper-chain的pallet的迁移至solana

1. adsc 需要在solana上发行spl-token代币来迁移adsc 代币
2. credit 信用数据迁移到solana上，数据量比较大，需要分片。用户需要确认solana地址。
3. operation 功能
   a. 桥的功能，需要把deeper-chain的dpr转到solana上
   b. ezc  待定
   c. 质押的线性释放，需要智能合约中根据天数，让用户自行领取
   d. 用户代币的冻结等操作，可能的方案是把solana的dpr合约的authority 更改为本合约
4. staking 质押迁移到solana上
5. tips 独立网页，发布任务需求？
6. Uniques 重新在solana上发行spl-token的nft
7. Ethereum 以太坊兼容的链，跟operation的ezc 结合考虑


. credit-accumulate 功能放在proxy上，proxy收集设备的credit增减信息以后，批量发送到solana合约中修改
. deeper-node 功能放在proxy上，proxy收集设备的在线信息，把credit 减少的消息，批量发送到solana合约中修改
. Micropayment 考虑proxy累积，然后发送到合约处理



```
      +-------+          +-------------------+
      |       |          |                   |
      | Users |--------->| Solana Contract   |
      |       |          |                   |
      +---+---+          +---------+---------+
          |                        ^
          |                        |
          |                        |
          v                        |
      +---+---+                    |
      |       |                    |
      | Proxy |--------------------+                    |
      |       |                    
      +---+---+                    
        
```

