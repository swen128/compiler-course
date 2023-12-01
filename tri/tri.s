        default rel
        section .text
        global entry
entry:
        mov rbx, 36             ; the "input"
        mov rax, 0
;;; tri: a recursive function for computing nth
;;; triangular number, where n is given in rbx.
tri:
        cmp rbx, 0              ; if rbx = 0, done
        je done
        add rax, rbx            ; result is rbx+tri(rbx-1)
        sub rbx, 1
        call tri
        ret
done:                          ; jump here for base case
        ret
