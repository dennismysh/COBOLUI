       IDENTIFICATION DIVISION.
       PROGRAM-ID. TODO-LIST.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  TASK-1          PIC X(40) VALUE "Buy groceries".
           05  TASK-2          PIC X(40) VALUE "Write report".
           05  TASK-3          PIC X(40) VALUE "Call dentist".
           05  NEW-TASK        PIC X(40) VALUE "Enter task here".
           05  TASK-COUNT      PIC 9(2) VALUE 3.
           05  DONE-COUNT      PIC 9(2) VALUE 0.
           05  STATUS-1        PIC X(10) VALUE "PENDING".
               88  TASK-1-DONE  VALUE "DONE".
               88  TASK-1-TODO  VALUE "PENDING".
           05  STATUS-2        PIC X(10) VALUE "PENDING".
               88  TASK-2-DONE  VALUE "DONE".
               88  TASK-2-TODO  VALUE "PENDING".
           05  STATUS-3        PIC X(10) VALUE "PENDING".
               88  TASK-3-DONE  VALUE "DONE".
               88  TASK-3-TODO  VALUE "PENDING".
           05  SUMMARY-TEXT    PIC X(60) VALUE "3 tasks".
           05  STATUS-MSG      PIC X(60) VALUE "Manage your tasks".

       SCREEN SECTION.
       01  LIST-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(30) VALUE "Todo List".
           05  TASKS.
               10  T1-NAME     PIC X(40) USING TASK-1.
               10  T1-STATUS   PIC X(10) USING STATUS-1.
               10  T2-NAME     PIC X(40) USING TASK-2.
               10  T2-STATUS   PIC X(10) USING STATUS-2.
               10  T3-NAME     PIC X(40) USING TASK-3.
               10  T3-STATUS   PIC X(10) USING STATUS-3.
           05  ACTIONS.
               10  DONE1-BTN   VALUE "Complete-1" ON-ACTION PERFORM MARK-TASK-1.
               10  DONE2-BTN   VALUE "Complete-2" ON-ACTION PERFORM MARK-TASK-2.
               10  DONE3-BTN   VALUE "Complete-3" ON-ACTION PERFORM MARK-TASK-3.
               10  COUNT-BTN   VALUE "Refresh" ON-ACTION PERFORM COUNT-DONE.
           05  NAV-AREA.
               10  ADD-NAV     VALUE "Add-Task" GO-TO-SCREEN ADD-SCREEN.
           05  SUMMARY-AREA.
               10  SUM-DISP    PIC X(60) USING SUMMARY-TEXT.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       01  ADD-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(30) VALUE "Add-Task".
           05  INPUT-AREA.
               10  TASK-INPUT  PIC X(40) USING NEW-TASK.
           05  ACTIONS.
               10  SAVE-BTN    VALUE "Save" ON-ACTION PERFORM SAVE-NEW-TASK.
               10  BACK-BTN    VALUE "Back" GO-TO-SCREEN LIST-SCREEN.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       MARK-TASK-1.
           SET TASK-1-DONE TO TRUE.
           PERFORM COUNT-DONE.
           MOVE "Task 1 done" TO STATUS-MSG.

       MARK-TASK-2.
           SET TASK-2-DONE TO TRUE.
           PERFORM COUNT-DONE.
           MOVE "Task 2 done" TO STATUS-MSG.

       MARK-TASK-3.
           SET TASK-3-DONE TO TRUE.
           PERFORM COUNT-DONE.
           MOVE "Task 3 done" TO STATUS-MSG.

       COUNT-DONE.
           MOVE 0 TO DONE-COUNT.
           IF STATUS-1 = "DONE"
               ADD 1 TO DONE-COUNT
           END-IF.
           IF STATUS-2 = "DONE"
               ADD 1 TO DONE-COUNT
           END-IF.
           IF STATUS-3 = "DONE"
               ADD 1 TO DONE-COUNT
           END-IF.
           STRING TASK-COUNT DELIMITED BY SIZE
                  " tasks, " DELIMITED BY SIZE
                  DONE-COUNT DELIMITED BY SIZE
                  " done" DELIMITED BY SIZE
           INTO SUMMARY-TEXT.

       SAVE-NEW-TASK.
           MOVE NEW-TASK TO TASK-3.
           MOVE "PENDING" TO STATUS-3.
           ADD 1 TO TASK-COUNT.
           PERFORM COUNT-DONE.
           MOVE "Task saved" TO STATUS-MSG.
