# Android Control Zen (for `amc`)

Assume less. Observe more.  
One action, one check.  
Learn the environment before pushing goals.  
For long runs, keep reporting and validating.  
Failure is normal; blind execution is not.

## 0. Opening Move (Environment First)
Always start with:
1. `preflight`
2. `observe top`
3. `observe screenshot`

If these are unstable, do not proceed.

## 1. Core Loop (Step by Step)
Every action follows the same loop:
1. Execute one action only.
2. Observe immediately (`top` + `screenshot`).
3. Decide the next action from observed state.

No blind action chains.

## 2. Long-Horizon Operations
For multi-step tasks, report in short intervals:
- Every 1-3 steps: what was done, what was observed, what is next.
- If state is uncertain: stop, observe, then continue.

## 3. Validation Order
Validate in this order:
1. Service availability (`preflight`)
2. Foreground state (`observe top`)
3. Visual state (`observe screenshot`)

A command returning `OK` is not enough without state evidence.

## 4. Destructive Action Rule
Keep destructive actions (`stop`/`force-stop`) at the end.  
After running them, treat environment as changed and restart from Section 0.

## 5. Minimal Template
```bash
# action
amc --base-url "$BASE_URL" --token "$TOKEN" <command>
# observe
amc --base-url "$BASE_URL" --token "$TOKEN" observe top
amc --base-url "$BASE_URL" --token "$TOKEN" observe screenshot --max-dim 720 --quality 70
```
