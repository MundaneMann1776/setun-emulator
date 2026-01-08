; Simple Setun test - data follows code immediately
; This layout works with the basic assembler

; Code starts at address 0
    LDA 4       ; Load value at address 4 (first data word after code)
    ADD 5       ; Add value at address 5 (second data word)
    STA 6       ; Store result to address 6
    HLT         ; Halt - result should be 59 in S

; Data immediately follows (at addresses 4, 5, 6)
    DAT 42      ; Address 4: First operand
    DAT 17      ; Address 5: Second operand
    DAT 0       ; Address 6: Result location
