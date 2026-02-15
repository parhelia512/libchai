use super::变异;
use crate::contexts::default::{
    默认上下文, 默认决策, 默认决策变化, 默认决策空间, 默认安排
};
use crate::optimizers::决策;
use crate::元素图;
use crate::错误;
use rand::seq::{IndexedRandom, IteratorRandom};
use rand::{random_range, rng};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::VecDeque;

pub struct 默认操作 {
    决策空间: 默认决策空间,
    元素图: 元素图,
}

#[skip_serializing_none]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct 变异配置 {
    pub random_move: f64,
    pub random_swap: f64,
    pub random_full_key_swap: f64,
}

pub const DEFAULT_MUTATE: 变异配置 = 变异配置 {
    random_move: 0.9,
    random_swap: 0.09,
    random_full_key_swap: 0.01,
};

impl 变异 for 默认操作 {
    type 决策 = 默认决策;
    fn 变异(&mut self, 决策: &mut Self::决策) -> <默认决策 as 决策>::变化 {
        let mut 变化 = self.随机移动元素(决策);
        self.传播(&mut 变化, 决策);
        变化
    }
}

// 默认的问题实现，使用配置文件中的约束来定义各种算子
impl 默认操作 {
    pub fn 新建(上下文: &默认上下文) -> Result<Self, 错误> {
        Ok(Self {
            决策空间: 上下文.决策空间.clone(),
            元素图: 上下文.元素图.clone(),
        })
    }

    fn 传播(&self, 变化: &mut <默认决策 as 决策>::变化, 决策: &mut 默认决策) {
        // 初始化队列
        let mut 队列 = VecDeque::new();
        for 元素 in 变化
            .增加元素
            .iter()
            .chain(变化.减少元素.iter())
            .chain(变化.移动元素.iter())
        {
            for 下游元素 in self.元素图.get(元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(下游元素.clone());
                }
            }
        }
        // 传播直到队列为空
        let mut iters = 0;
        while !队列.is_empty() {
            iters += 1;
            if iters > 100 {
                panic!("传播超过 100 次仍未结束，可能出现死循环");
            }
            let 元素 = 队列.pop_front().unwrap();
            let 当前安排 = 决策.元素[元素];
            let mut 合法 = false;
            let mut 新安排列表 = vec![];
            for 条件安排 in &self.决策空间.元素[元素] {
                if 决策.允许(条件安排) {
                    if 条件安排.安排 == 当前安排 {
                        合法 = true;
                        break;
                    }
                    新安排列表.push(条件安排.安排.clone());
                }
            }
            if !合法 {
                if 新安排列表.is_empty() {
                    panic!("没有合法的安排，传播失败");
                } else {
                    let 新安排 = *新安排列表.choose(&mut rng()).unwrap();
                    if let 默认安排::未选取 = 当前安排 {
                        变化.增加元素.push(元素);
                    } else if let 默认安排::未选取 = 新安排 {
                        变化.减少元素.push(元素);
                    } else {
                        变化.移动元素.push(元素);
                    }
                    决策.元素[元素] = 新安排;
                }
            }
            for 下游元素 in self.元素图.get(&元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(*下游元素);
                }
            }
        }
    }

    pub fn 随机移动元素(&self, 决策: &mut 默认决策) -> 默认决策变化 {
        let mut rng = rng();
        const MAX_TRIES: usize = 100;
        for _ in 0..MAX_TRIES {
            let 元素 = (0..决策.元素.len()).choose(&mut rng).unwrap();
            let 当前安排 = 决策.元素[元素];
            // 蓄水池抽样
            let mut 下一个安排 = None;
            let mut count = 0;
            for 条件安排 in &self.决策空间.元素[元素] {
                if 条件安排.安排 != 当前安排 && 决策.允许(条件安排) {
                    count += 1;
                    if random_range(0..count) == 0 {
                        下一个安排 = Some(&条件安排.安排);
                    }
                }
            }
            if let Some(下一个安排) = 下一个安排 {
                决策.元素[元素] = *下一个安排;
                let mut 增加元素 = vec![];
                let mut 减少元素 = vec![];
                let mut 移动元素 = vec![];
                if let 默认安排::未选取 = 当前安排 {
                    增加元素.push(元素);
                } else if let 默认安排::未选取 = 下一个安排 {
                    减少元素.push(元素);
                } else {
                    移动元素.push(元素);
                }
                return 默认决策变化::新建(增加元素, 减少元素, 移动元素);
            }
        }
        默认决策变化::不变()
    }
}
