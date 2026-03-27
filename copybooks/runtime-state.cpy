      *> runtime-state.cpy - Live runtime state
      *> Maps Rust: RuntimeState (HashMap<String, String>)
      *> plus current-screen, focus, app-running flag

       01  WS-RUNTIME-STATE.
           05  WS-CURRENT-SCREEN-IDX PIC 9(2) VALUE 1.
           05  WS-FOCUS-IDX          PIC 9(4) VALUE 1.
           05  WS-APP-RUNNING        PIC 9(1) VALUE 1.
               88  APP-IS-RUNNING    VALUE 1.
               88  APP-IS-STOPPED    VALUE 0.
           05  WS-RECURSION-DEPTH    PIC 9(4) VALUE 0.
           05  WS-LOOP-COUNTER       PIC 9(6) VALUE 0.

      *> Live variable values (parallel to state table)
       01  WS-LIVE-VALUES.
           05  WS-LIVE-COUNT         PIC 9(4) VALUE 0.
           05  WS-LIVE-ENTRY OCCURS 200 TIMES.
               10  LIVE-VAR-NAME     PIC X(30).
               10  LIVE-VAR-VALUE    PIC X(80).
