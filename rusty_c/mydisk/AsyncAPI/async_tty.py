def tty_read():
  while True:
    c = uart_read()
    if c == -1:
      enq(tty_wq, curproc)
      scheduler()
    else:
      return c
    
def tty_intr():
  p = deq(tty_wq)
  if p is not None:
    enq(runq, p)