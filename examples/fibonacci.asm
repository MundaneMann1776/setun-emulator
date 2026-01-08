; Fibonacci sequence on Setun
; Computes first few Fibonacci numbers
; Result stored in S register

; Memory layout:
; Address 50: Previous Fibonacci (initially 0)
; Address 51: Current Fibonacci (initially 1)
; Address 52: Counter (iterations remaining)

START:
    ; Initialize: F(0)=0, F(1)=1, count=8
    LDA 60          ; Load initial prev (0)
    STA 50          ; Store to prev location
    LDA 61          ; Load initial curr (1)
    STA 51          ; Store to curr location
    LDA 62          ; Load counter (8 iterations)
    STA 52          ; Store counter
    
LOOP:
    ; Check if done
    LDA 52          ; Load counter
    JZ DONE         ; If zero, we're done
    
    ; Compute next = prev + curr
    LDA 50          ; Load prev
    ADD 51          ; Add curr -> next is in S
    
    ; Shift: prev = curr, curr = next
    XCHG 51         ; Swap S with curr: S now has old curr, [51] has new next
    STA 50          ; Store old curr to prev
    
    ; Decrement counter
    LDA 52          ; Load counter
    ADD 63          ; Add -1 (subtract 1)
    STA 52          ; Store counter
    
    JMP LOOP        ; Continue

DONE:
    LDA 51          ; Load final Fibonacci number to S
    HLT             ; Halt with result in S

; Constants and data
    ORG 60
    DAT 0           ; Initial prev
    DAT 1           ; Initial curr
    DAT 8           ; Counter (8 iterations -> F(8) = 21)
    DAT -1          ; Decrement constant
