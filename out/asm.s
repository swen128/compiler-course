	default rel
	section .text
	global entry
entry:
	mov rbx, 36
tri:
	cmp rbx, 0
	je end
	push rbx
	sub rbx, 1
	call tri
	pop rbx
	add rax, rbx
	ret
end:
	mov rax, 0
	ret
