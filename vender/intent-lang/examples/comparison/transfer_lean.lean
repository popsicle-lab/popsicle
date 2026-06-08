-- ============================================================
-- 银行转账 — Lean 4 实现
-- ============================================================

-- 1. 定义类型
structure Account where
  balance : Int
  owner : String
  active : Bool

-- 2. 定义转账操作（Lean 需要你写出具体实现）
def transfer (sender receiver : Account) (amount : Int) : Account × Account :=
  ( { sender with balance := sender.balance - amount },
    { receiver with balance := receiver.balance + amount } )

-- 3. 前置条件（需要手动拆分为独立定义）
def transferPre (sender receiver : Account) (amount : Int) : Prop :=
  amount > 0 ∧ sender.balance ≥ amount ∧ sender.active ∧ receiver.active

-- 4. 后置条件
def transferPost (sender receiver sender' receiver' : Account) (amount : Int) : Prop :=
  sender'.balance = sender.balance - amount ∧
  receiver'.balance = receiver.balance + amount ∧
  sender'.balance ≥ 0

-- 5. 定理：转账满足后置条件（需要手写证明策略）
theorem transfer_correct (sender receiver : Account) (amount : Int)
    (h : transferPre sender receiver amount) :
    let (s', r') := transfer sender receiver amount
    transferPost sender receiver s' r' amount := by
  unfold transferPre at h
  unfold transfer transferPost
  simp
  omega

-- 6. 定理：总额守恒
theorem transfer_preserves_total (sender receiver : Account) (amount : Int)
    (h : transferPre sender receiver amount) :
    let (s', r') := transfer sender receiver amount
    s'.balance + r'.balance = sender.balance + receiver.balance := by
  unfold transfer
  simp
  omega

-- ============================================================
-- 用户认证 — Lean 4 实现
-- ============================================================

inductive Role where
  | admin | editor | viewer

structure User where
  id : Nat
  role : Role
  authenticated : Bool
  loginAttempts : Nat
  locked : Bool

structure Resource where
  ownerId : Nat
  public : Bool

-- 登录操作：需要写出完整的状态转换逻辑
def login (user : User) (passwordCorrect : Bool) : User :=
  if user.locked then user  -- 调用者应检查前置条件
  else if passwordCorrect then
    { user with authenticated := true, loginAttempts := 0, locked := false }
  else
    let attempts := user.loginAttempts + 1
    { user with authenticated := false,
                loginAttempts := attempts,
                locked := attempts ≥ 5 }

-- 前置条件
def loginPre (user : User) : Prop := ¬user.locked

-- 定理：锁定用户无法通过前置条件
theorem locked_user_cannot_login (user : User) (h : user.locked) :
    ¬(loginPre user) := by
  unfold loginPre
  simp [h]

-- 定理：密码正确 → 认证成功（需要逐步拆解证明）
theorem login_success (user : User) (h1 : loginPre user) :
    (login user true).authenticated = true := by
  unfold loginPre at h1
  unfold login
  simp [h1]

-- 定理：失败 5 次 → 锁定（证明更复杂，需要归纳）
-- 这里省略，实际需要对 loginAttempts 做归纳证明...

-- ============================================================
-- 排序规范 — Lean 4 实现
-- ============================================================

def isSorted : List Int → Prop
  | [] => True
  | [_] => True
  | a :: b :: rest => a ≤ b ∧ isSorted (b :: rest)

-- Lean 中排列用 List.Perm（标准库提供）
def sortSpec (input output : List Int) : Prop :=
  isSorted output ∧ input.Perm output

-- 定理：排序幂等性
theorem sort_idempotent (sort : List Int → List Int)
    (h_correct : ∀ l, sortSpec l (sort l)) :
    ∀ l, sort (sort l) = sort l := by
  intro l
  -- 需要证明：已排序的列表再排序不变
  -- 这需要额外引理 + 较长的证明...
  sorry  -- 非平凡证明，此处省略
