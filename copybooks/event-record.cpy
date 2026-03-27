      *> event-record.cpy - Single event record
      *> Maps Rust: EventRecord { event_type, target, payload }

       01  WS-EVENT-RECORD.
           05  EVENT-TYPE-CODE       PIC 9(1) VALUE 0.
               88  EVT-IS-CLICK         VALUE 1.
               88  EVT-IS-INPUT         VALUE 2.
               88  EVT-IS-NAVIGATE      VALUE 3.
               88  EVT-IS-QUIT          VALUE 4.
               88  EVT-IS-RESIZE        VALUE 5.
           05  EVENT-TARGET-NAME     PIC X(30).
           05  EVENT-PAYLOAD         PIC X(80).
           05  EVENT-PENDING         PIC 9(1) VALUE 0.
               88  EVT-HAS-EVENT     VALUE 1.
               88  EVT-NO-EVENT      VALUE 0.
