      *> render-types.cpy - Terminal rendering work areas
      *> Layout calculation and ANSI output buffers

       01  WS-RENDER-STATE.
      *> Terminal dimensions
           05  WS-TERM-LINES         PIC 9(4) VALUE 24.
           05  WS-TERM-COLS          PIC 9(4) VALUE 80.
      *> Current cursor position during render
           05  WS-CURRENT-LINE       PIC 9(4) VALUE 1.
           05  WS-CURRENT-COL        PIC 9(4) VALUE 1.
      *> Layout scratch
           05  WS-LAYOUT-DEPTH       PIC 9(2) VALUE 0.
           05  WS-AVAIL-WIDTH        PIC 9(4) VALUE 0.
           05  WS-AVAIL-HEIGHT       PIC 9(4) VALUE 0.
           05  WS-CHILD-HEIGHT       PIC 9(4) VALUE 0.
      *> ANSI output buffer (one line at a time)
           05  WS-ANSI-BUFFER        PIC X(1024).
           05  WS-ANSI-BUFFER-LEN    PIC 9(4) VALUE 0.
      *> Color state tracking
           05  WS-ACTIVE-FG          PIC 9(1) VALUE 9.
           05  WS-ACTIVE-BG          PIC 9(1) VALUE 9.
      *> Redraw flag
           05  WS-NEEDS-REDRAW       PIC 9(1) VALUE 1.
               88  RENDER-NEEDS-REDRAW   VALUE 1.
               88  RENDER-IS-CLEAN       VALUE 0.
      *> Input key buffer
           05  WS-KEY-BUFFER         PIC X(10).
           05  WS-KEY-BUFFER-LEN     PIC 9(2) VALUE 0.
