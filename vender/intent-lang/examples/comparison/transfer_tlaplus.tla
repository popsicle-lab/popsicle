---------------------------- MODULE Transfer ----------------------------
(* ================================================================== *)
(* 银行转账 — TLA+ 实现                                                *)
(* ================================================================== *)

EXTENDS Integers, Sequences

CONSTANTS Accounts, InitBalances

VARIABLES balances, locked

(* ------ 类型不变量 ------ *)
TypeOK ==
    /\ balances \in [Accounts -> Nat]
    /\ locked \in [Accounts -> BOOLEAN]

(* ------ 初始状态 ------ *)
Init ==
    /\ balances = InitBalances
    /\ locked = [a \in Accounts |-> FALSE]

(* ------ 转账动作 ------ *)
(* TLA+ 用 primed 变量 (x') 表示下一状态，这点和 intent-lang 相同 *)
Transfer(sender, receiver, amount) ==
    /\ sender /= receiver
    /\ amount > 0
    /\ balances[sender] >= amount
    /\ ~locked[sender]
    /\ ~locked[receiver]
    /\ balances' = [balances EXCEPT
        ![sender] = balances[sender] - amount,
        ![receiver] = balances[receiver] + amount]
    /\ UNCHANGED locked

(* ------ 有 Bug 的转账（演示 model checker 检测） ------ *)
TransferBuggy(sender, receiver, amount) ==
    /\ sender /= receiver
    /\ amount > 0
    /\ balances[sender] >= amount
    /\ balances' = [balances EXCEPT
        ![sender] = balances[sender] - amount - 1,  \* Bug: 多扣了 1
        ![receiver] = balances[receiver] + amount]
    /\ UNCHANGED locked

(* ------ Next 状态转换 ------ *)
Next ==
    \E s, r \in Accounts, a \in 1..100 :
        Transfer(s, r, a)

(* ------ 不变量：余额非负 ------ *)
BalanceNonNegative ==
    \A a \in Accounts : balances[a] >= 0

(* ------ 不变量：总额守恒 ------ *)
TotalPreserved ==
    LET Total(b) == LET S == DOMAIN b
                     IN  (* 需要递归求和，TLA+ 中较繁琐 *)
                         TRUE  \* 简化处理
    IN TRUE

(* ------ 规范 ------ *)
Spec == Init /\ [][Next]_<<balances, locked>>

(* ------ 要验证的性质 ------ *)
THEOREM Spec => []BalanceNonNegative

=========================================================================

(* ================================================================== *)
(* 用户认证 — TLA+ 实现                                                *)
(* ================================================================== *)
---------------------------- MODULE Auth --------------------------------

EXTENDS Integers

CONSTANTS Users, MaxAttempts

VARIABLES auth, attempts, isLocked

AuthTypeOK ==
    /\ auth \in [Users -> BOOLEAN]
    /\ attempts \in [Users -> Nat]
    /\ isLocked \in [Users -> BOOLEAN]

AuthInit ==
    /\ auth = [u \in Users |-> FALSE]
    /\ attempts = [u \in Users |-> 0]
    /\ isLocked = [u \in Users |-> FALSE]

(* TLA+ 中登录是一个"动作"，需要描述完整的状态转换 *)
Login(user, passwordCorrect) ==
    /\ ~isLocked[user]
    /\ IF passwordCorrect
       THEN /\ auth' = [auth EXCEPT ![user] = TRUE]
            /\ attempts' = [attempts EXCEPT ![user] = 0]
            /\ isLocked' = isLocked
       ELSE /\ auth' = [auth EXCEPT ![user] = FALSE]
            /\ attempts' = [attempts EXCEPT ![user] = attempts[user] + 1]
            /\ isLocked' = [isLocked EXCEPT
                ![user] = (attempts[user] + 1 >= MaxAttempts)]

(* 性质：锁定的用户不能登录 *)
LockedCannotLogin ==
    \A u \in Users :
        isLocked[u] => ~ENABLED(\E p \in BOOLEAN : Login(u, p))

AuthNext ==
    \E u \in Users, p \in BOOLEAN :
        Login(u, p)

AuthSpec == AuthInit /\ [][AuthNext]_<<auth, attempts, isLocked>>

THEOREM AuthSpec => []LockedCannotLogin

=========================================================================
