# OSDev Wiki — volledige bronteksten, deel 10

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w003533"></a>

## W003533 — User:Johnburger/Demo/Ints/IRQs

Bron-ID: W003533
Namespace: User (2)
Revisie: 21380 | Datum: 2017-08-28T21:24:26Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/IRQs
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FIRQs&oldid=21380

````text
Like <code>[[User:Johnburger/Demo/Ints/Vectors|Vectors]]</code>, this code is merely the entry points for each of the Interrupt ReQuest handlers. They all perform the same steps:
* <code>PUSH</code> a pseudo-fault code onto the stack, to make the stack Frame the same for the [[User:Johnburger/Demo/Ints/Fault|Fault]] handler;
* <code>PUSH</code> the Interrupt number onto the stack;
* <code>JMP</code> to a routine to acknowledge the [[User:Johnburger/PIC|PIC(s)]];
* <code>JMP</code> to the default [[User:Johnburger/Demo/Ints/Fault|Fault]] handler to display the registers.

One of the steps that the Protected Mode initialisation does is to register new handlers for the [[User:Johnburger/Demo/Ints/Timer|Timer]] and [[User:Johnburger/Demo/Ints/Key|Keyboard]], replacing these default ones.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/IRQs.inc</code> ==
<source lang="asm">
;
; Ints/IRQs.inc
;

; This file holds the entry points for all of the Interrupt ReQuests (IRQs).
; They all jump to the same handler, but not before saving away the number
; of the interrupt - oh, and a fault code since these should not happen!

; To save space, the handler has been put in the middle of the vectors, to
; allow the assembler to use shorter JMPs.
Ints.IRQ0:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    32      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ1:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    33      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ2:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    34      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ3:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    35      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ4:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    36      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ5:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    37      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ6:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    38      ; Interrupt number
                JMP             Ints.PIC.A
Ints.IRQ7:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    39      ; Interrupt number
                JMP             Ints.PIC.A

; Acknowledge Slave PIC (first)
Ints.PIC.B:
                PUSH            EAX
                MOV             AL,Dev.PIC.Cmd.EOI
                OUT             Dev.PIC.B.Cmd,AL
                POP             EAX
; Acknowledge Master PIC
Ints.PIC.A:
                PUSH            EAX
                MOV             AL,Dev.PIC.Cmd.EOI
                OUT             Dev.PIC.A.Cmd,AL
                POP             EAX
; All IRQs jump to the default Fault handler
                JMP             Ints.Fault

Ints.IRQ8:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    40      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ9:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    41      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ10:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    42      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ11:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    43      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ12:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    44      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ13:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    45      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ14:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    46      ; Interrupt number
                JMP             Ints.PIC.B
Ints.IRQ15:
                PUSH   DWORD    0       ; Pseudo-fault code
                PUSH   DWORD    47      ; Interrupt number
                JMP             Ints.PIC.B
</source>
````

<a id="w003534"></a>

## W003534 — User:Johnburger/Demo/Ints/Timer

Bron-ID: W003534
Namespace: User (2)
Revisie: 21381 | Datum: 2017-08-28T21:24:49Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FTimer&oldid=21381

````text
The Timer interrupt handler doesn't have to do much: the Periodic Interval Timer chip doesn't need any steps to acknowledge the interrupt it generated. That means that the handler just has to:
* Save the registers it's going to use;
* Acknowledge the interrupt in the [[User:Johnburger/PIC|PIC]];
* <code>CALL</code> the generic [[User:Johnburger/Demo/Ints/Util|Task Switching]] code;
* Restore the registers;
* Return to the interrupted code.

Note that if a Task Switch occurred, the code that was resumed in the last step won't be the actual code that was interrupted: that code has been paused until the Task Scheduler gets back around to it.

== <code>Timer.inc</code> ==
<source lang="asm">
;
; Ints/Timer.inc
;

; This module implements the Timer interrupt handler.
; It simply performs some housekeeping and then calls the generic Task
; Switching code.

Ints.Timer:
                PUSH            EAX             ; Need this register

                MOV             AL,Dev.PIC.Cmd.EOI ; Acknowledge interrupt in PIC
                OUT             Dev.PIC.A.Cmd,AL   ; (Note interrupts still off)

                CALL            Ints.Switch     ; Switch Task

                POP             EAX
                IRETD                           ; And return where we left off
</source>
````

<a id="w003535"></a>

## W003535 — User:Johnburger/Demo/Ints/Key

Bron-ID: W003535
Namespace: User (2)
Revisie: 21426 | Datum: 2017-08-29T09:22:18Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Key
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FKey&oldid=21426

````text
This keyboard interrupt handler reads the keycode from the keyboard device and stores it away in the Global Data segment. It also takes the time to display the Hex code for the keycode on the screen.

The code as it stands doesn't display [[User:Johnburger/Demo/Dev/Key#Multi-code_Keys|multi-code]] keys - the subsequent codes overwrite the previous ones, and it's not obvious which keys are multi-code. The solution to this is left as an exercise for the reader!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/Key.inc</code> ==
<source lang="asm">
;
; Ints/Key.inc
;

; This module implements the Keyboard interrupt handler.
; It simply gets the scan-code for the key and stores it into Data.Key.Code.
; This is a one-byte buffer - if you miss it before the next one, it's gone!
; Of course, not much is happening on this system, so it's unlikely it'll get
; missed.
;
; Note that the value stored is a scan-code, which is different from what you
; get from a getc() call - that's ASCII. Scan-codes are whatever the keyboard
; wants to generate for each key, and there's nothing obvious about their
; arrangement - so I've added code that displays the Hex value of the scan-code
; on the screen on every interrupt.
;
; Also note that the keyboard generates one interrupt when the key is pressed,
; and a second (with the same scan-code but with the high bit set) when the key
; is released. In this way the keyboard handler can keep track of which keys are
; currently being held down. If the key isn't released after a short while,
; another scan-code will be generated, again with the high bit clear.
;
; What isn't obvious with the display code as it is, is that for some keys there
; are two (or more) interrupts generated. The original PC had one set of number
; and arrow keys, while the Extended keyboard duplicated these with a keypad.
; For older systems these new keys had to generate the same scan-codes as the
; old ones, but newer systems wanted to distinguish between the two. So the new
; keys were designed to prefix the old scan-code with a "magic" keypress that
; the old systems didn't know about, so simply ignored.
;
; Needless to say, extending the functionality is left as an exercise for the
; reader!

Ints.Key:
                PUSH            EAX             ; Need these registers
                PUSH            ES              ; For .Hex's STOSW later...

                ; Point to Global Data
                MOV             AX,Selector(GDT.Data, GDT, RPL0)
                MOV             ES,AX

                IN              AL,Dev.Key.Data ; Get character from keyboard
                MOV             [ES:Data.Key.Code],AL ; Store in global Data

                MOV             AH,AL            ; Save to show

                MOV             AL,Dev.PIC.Cmd.EOI ; Can now acknowledge PIC
                OUT             Dev.PIC.A.Cmd,AL

                CMP             AH,Dev.Key.Pause ; Also labeled "Break"...
                JNE             .Continue        ; Don't Break?
                STI                              ; Do! So need interrupts back on
                DEBUG                            ; (Debugger uses keyboard)
.Continue:
                CALL            .Show           ; Show scan-code on screen

                POP             ES              ; That was easy!
                POP             EAX
                IRETD

;-------------------------------------------------------------------------------
; This function shows the scan-code (in AL) on the screen

.Show:

; .Key.Pos is chosen as the keyboard IRQ's screen position - but the Hex is two
; characters wide. Luckily, the next IRQ (IRQ2) is the Cascade, so it never fires!
.Key.Pos        EQU             33 * 2          ; Location on screen to show Key
.Key.Colour     EQU             Dev.VGA.CyanBack | Dev.VGA.Black

                PUSH            ECX             ; Need these
                PUSH            EDX
                PUSH            EDI  

                ; Now point to screen
                MOV             DI,Selector(GDT.VGA, GDT, RPL0)
                MOV             ES,DI

                MOV             DL,AH           ; Code to Hexify
                MOV             AH,.Key.Colour  ; Colour to use
                MOV             ECX,2           ; Only 2 digits
                MOV             EDI,.Key.Pos    ; Position to show it
                CALL            Ints.Hex

                POP             EDI             ; Done!
                POP             EDX
                POP             ECX
                RET
</source>
````

<a id="w003536"></a>

## W003536 — User:Johnburger/Demo/Ints/Util

Bron-ID: W003536
Namespace: User (2)
Revisie: 21444 | Datum: 2017-08-29T09:32:07Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Util
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FUtil&oldid=21444

````text
There are four utility functions and a useful table that different interrupt handlers may want to use.

== <code>Ints.Switch</code> ==
This function switches to the next available Task. It does it by getting the current Task Register, then looking past that one in the GDT for the next available TSS. If it reaches the end of the GDT it starts from the beginning again. If it finds a different one, it <code>JMP</code>s to it, invoking a Task Switch. Otherwise, it simply returns.

Note that there are some TSSes that should NOT be switched to: in particular, the ones that are waiting around to handle any special Fault exceptions. To flag those as special, I make use of the <code>[[User:Johnburger/Demo/x86/Desc|x86.Desc.Gran.Avail]]</code> bit in the Descriptor: Intel reserved that bit for system use, and that is what I want to use it for - with TSSes, at least.

For that matter, special TSSes shouldn't be switched away from either. If they were, there'd be bouncing balls all over the carefully-crafted Hex screen contents...

== <code>Ints.System</code> ==
I use the <code>System.TSS</code> flag to mark a TSS as being "special", as described above. But I also need to be able to temporarily mark a normal TSS as special so that I can debug it. These two routines handle this.

=== <code>.Init</code> ===
This routine marks the current Task (as indicated in the <code>TR</code>) as a <code>System.TSS</code> - unless it already is one. It returns a flag to indicate what it did, which needs to be saved and passed back to <code>.Done</code> below when the TSS is finished with.

=== <code>.Done</code> ===
This routine undoes whatever <code>.Init</code> above did.

== <code>Ints.Hex</code> ==
This routine takes a number of parameters:
* Value to display;
* Number of Hex digits to display;
* Where on the screen to display it;
* What colour to use;

and displays the Hex representation of the Value, updating the screen position.

== <code>Ints.Name</code> ==
I wanted both the Fault handler and the Debugger to display register contents, but I wanted a minimal overhead. So I designed a simple table with five fields:
* A one-byte screen location for the Debug window (the Fault window just displays them one below the other);
* A four-character register name, with internal padding for alignment:
* A one-byte number of Hex digits to display;
* A one-byte offset on the stack for the register's location;
* A one-byte offset within a TSS for the register's location.

It turns out that even when displaying a TSS' contents, some of the values of interest are actually still on the stack. The code can distinguish those, though: the low-down offsets are stack offsets rather than TSS ones.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/Util.inc</code> ==
<source lang="asm">
;
; Ints/Util.inc
;

; This module holds Utility functions for various Fault handlers to use

;...............................................................................
; This function switches to the next available Task. Note that it assumes that
; interrupts are OFF!
;
; It cycles through the Global Descriptors looking for Idle TSSs. When it finds
; one, it JMPs to it - stopping the current Task, making it Idle for next time.
;
; Note that I use x86.Desc.Mem.Gran.Avail to flag a System TSS: it may be Idle
; now, but that'd be because nothing's using it yet -  a Fault handler is a
; good example of a System TSS waiting to be used.
;
; Also note that if the TSS that is currently running is a System TSS, I won't
; switch away from it either!

Ints.Switch:

.Type.Mask      EQU             x86.Desc.Type.Present | x86.Desc.Type.Sys | x86.Desc.Type.Type
.IdleTSS        EQU             Type.Sys(TSS, DPL0, 386) ; Target!

                STR             EAX             ; Get current TSS
                TEST            EAX,EAX         ; Switching Tasks yet?
                JZ              .End            ; No, so leave

                PUSH            EBX             ; Need some more registers
                PUSH            DS              ; And a Segment register

                MOV             EBX,EAX
                MOV     WORD    AX,Selector(GDT.Alias, GDT, RPL0)
                MOV             DS,AX           ; To point to the GDT

                ; I know that EAX holds a TSS - but is it a System TSS?
                TEST    BYTE    [EBX+x86.Desc.Mem.Gran],System.TSS
                JNZ             .Kill           ; Yes! Abort!

                PUSH            EDX             ; Now need this

                MOV             EDX,EBX         ; Starting point
.Loop:
                ADD             BX,x86.Desc_size     ; Look at next descriptor
                JZ              .Restart             ; Overflow!
                CMP             BX,[GDT.Alloc.Limit] ; Too far?
                JB              .Continue            ; Not yet...
.Restart:
                MOV             BX,GDT.Limit+1       ; Yes. Start at original Limit
.Continue:
                CMP             EBX,EDX         ; Looped back to here again?
                JE              .Finish         ; Yes, so nothing to switch to

                MOV             AL,[EBX+x86.Desc.Mem.Type] ; Get Descriptor Type
                AND             AL,.Type.Mask   ; Isolate interesting bits
                CMP             AL,.IdleTSS     ; Is it an Idle TSS?
                JNE             .Loop           ; No. Keep looking

                TEST    BYTE    [EBX+x86.Desc.Mem.Gran],System.TSS
                JNZ             .Loop           ; Ignore System TSSs too

                PUSH            EBX             ; Found! So JMP to it!
                PUSH            0               ; (Note that Offset is ignored)
                JMP     FAR     [ESP]           ; Invokes Task Switch
                ADD             ESP,8           ; When JMPs back, continues here
.Finish:
                POP             EDX
.Kill:
                POP             DS
                POP             EBX
.End:
                RET
;...............................................................................
; This function sets the current TSS to be a System TSS.
; Output: EAX = whether it modified the Task. Pass this in EBX to .Done
;         GS  = GDT
Ints.System.Init:
                MOV             AX,Selector(GDT.Alias, GDT, RPL0)
                MOV             GS,AX           ; Point to GDT

                STR             EAX             ; Get current Task
                TEST            EAX,EAX         ; Is there one?
                JZ              .End            ; No, so nothing to do
                TEST    BYTE    [GS:EAX+x86.Desc.Mem.Gran],System.TSS
                JNZ             .System         ; Already a System TSS!
                OR      BYTE    [GS:EAX+x86.Desc.Mem.Gran],System.TSS
                JMP             .End            ; Make it one temporarily
.System:
                XOR             EAX,EAX         ; Already a System TSS! Don't touch!
.End:
                RET
;...............................................................................
; This function reverts any change made by .Init (above)
; Input:  EBX = EAX output from .Init
;         GS  = GDT

Ints.System.Done:
                TEST            EBX,EBX         ; Was current TSS modified?
                JZ              .End            ; No, so just leave
                AND     BYTE    [GS:EBX+x86.Desc.Mem.Gran],~System.TSS
.End:
                RET
;...............................................................................
; This function displays a Hex value on the screen.
; Input:  AH  = Colour to use
;         ECX = Number of digits to display
;         EDX = Value to display
;         EDI = Screen position to display
;         ES  = Screen segment
; Output: AL, EDX modified. ECX zeroed. EDI preserved
Ints.Hex:
                STD                             ; Work backwards
                LEA             EDI,[EDI+ECX*2-2] ; Point to end of number
                PUSH            EDI             ; Save for end
.Loop:
                MOV             AL,DL           ; Get lowest byte
                AND             AL,0Fh          ; Isolate low nybble
                ADD             AL,'0'          ; Turn into ASCII
                CMP             AL,'9'          ; Too far?
                JBE             .NotHex         ; No
                ADD             AL,'A'-'9'-1    ; Yes, so turn into Hex
.NotHex:
                STOSW                           ; Store
                SHR             EDX,4           ; Shift in next nybble
                LOOP            .Loop           ; Loop
                CLD                             ; Assume forwards
                POP             EDI             ; Saved end
                ADD             EDI,2           ; Past last digit
                RET                             ; And return
;-------------------------------------------------------------------------------
; The following table lists the information for each register to show:
; Debug offset, Label, Num Hex chars, Stack offset, TSS offset

Ints.Names:
.Label          EQU             4  ; Width of label
;                              Debug,  Name , W,Stk,TSS
                DB                 0, ' Int', 2, 68, 20 ; On stack
                DB                 0, ' Err', 4, 72, 24 ; On stack
.Regs: ; Only registers from here
                DB              002h, ' EAX', 8, 64, 40
                DB              011h, ' EBX', 8, 52, 52
                DB              020h, ' ECX', 8, 60, 44
                DB              02Fh, ' EDX', 8, 56, 48
                DB              042h, ' ESI', 8, 40, 64
                DB              051h, ' EDI', 8, 36, 68
                DB              060h, ' EBP', 8, 44, 60
                DB              0DAh, ' ESP', 8, 48, 56
                DB              0C7h, ' EIP', 8, 76, 32
                DB              0C2h, '  CS', 4, 80, 76
                DB              082h, '  DS', 4, 32, 84
                DB              08Dh, '  ES', 4, 28, 72
                DB              098h, '  FS', 4, 24, 88
                DB              0A3h, '  GS', 4, 20, 92
                DB              0D5h, '  SS', 4, 16, 80
                DB              06Fh, 'Flag', 8, 84, 36
                DB              0AFh, 'PDBR', 8,  4, 28 ; Minimum NOT on stack
                DB              0F3h, 'Task', 4, 12, 12 ; On stack
                DB              0E9h, ' LDT', 4,  8, 96
.End            EQU             $
;    0000000000000000111111111111111122222222222222223333333333333333
;    0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF
;000:  _EAX:########  _EBX:########  _ECX:########  _EDX:########
;040:  _ESI:########  _EDI:########  _EBP:########  Flag:########
;080:  __DS:####  __ES:####  __FS:####  __GS:####   PDBR:########
;0C0:  __CS:####:######## __SS:####:######### _LDT:#### Task:####
</source>
````

<a id="w003537"></a>

## W003537 — User:Johnburger/Demo/User/User

Bron-ID: W003537
Namespace: User (2)
Revisie: 21361 | Datum: 2017-08-28T21:12:54Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/User/User
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FUser%2FUser&oldid=21361

````text
The User Task is less than 256 bytes. It didn't need to be big, but I did want it be both visual and obvious what was happening. I also wanted it to be flexible to the [[User:Johnburger/Demo/Demo.inc|configurable parameters]], even changing modes when not given much room at all - I wanted to be able to stress the system and literally max out the number of Tasks running 'simultaneously'.

Note, though, that precisely because the Task is so simple, upon first start-up all the Tasks may look like they're in "lock-step" - as though they're producing the same output at the same time. This is because they ''are'' the same code, starting from the ''same'' startup conditions. One way to diverge the different Tasks is to introduce more interrupts. (After giving the virtual screen the focus...) Just press lots of keys, or hold one down. What will happen is that Tasks will be "kicked awake" to update their ball position again, and different Tasks will be at different stages through their execution. Slowly but surely they will begin to show independence.

Also note that it is here where I define the two macros for the two System Calls that are supported.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#User_Task|User]]/User.inc</code> ==
<source lang="asm">
;
; User/User.inc
;

                SEGMENT         User  VSTART=0  ALIGN=16

                USE32

%define         User.Halt       INT 48
%define         User.Yield      INT 49

;-------------------------------------------------------------------------------
%include        "User/LDT.inc"
;-------------------------------------------------------------------------------
%include        "User/Data.inc"
;-------------------------------------------------------------------------------
%include        "User/Code.inc"
;-------------------------------------------------------------------------------
%include        "User/Frame.inc"
</source>
````

<a id="w003538"></a>

## W003538 — User:Johnburger/Demo/User/LDT

Bron-ID: W003538
Namespace: User (2)
Revisie: 21434 | Datum: 2017-08-29T09:29:27Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/User/LDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FUser%2FLDT&oldid=21434

````text
The LDT cannot be hard-coded at assembly time: there are lots of them, and they need to be built directly in memory by the [[User:Johnburger/Demo/Exec/Exec|Executive]]. But the [[User:Johnburger/Demo/User/Code|User code]] needs to know the values of the different segment Selectors, so a common definition is useful.

You'll also see that the Stack sizes are defined here. Multiple stacks? Yes. A requirement of Protected Mode programming is that User mode programs cannot affect the system - and if a rogue program nearly runs out of stack, the system won't have enough room to perform its critical functions, like servicing interrupts.

So every time the system needs to do a Supervisor mode operation, it switches to its own stack, as defined in the current [[User:Johnburger/Demo/Exec/User/TSS|TSS]]. In fact, up to four Stacks can be defined: one for each Privilege Level Ring. This Demonstrator only uses Rings 0 and 3 though.

Finally, the source has an assemble-time sanity check in it, to confirm a system-wide inviolate: I need both the Global and all Local Descriptor Tables to have an Alias to themselves, and I need them all to be at the same Selector index: <code>DT.Alias</code>. That way I can access them in other system code easily.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/User/User|User]]/LDT.inc</code> ==
<source lang="asm">
;
; User/LDT.inc
;

; This defines the LDT that the User program needs.

                SEGMENT         User.LDT  START=0  ALIGN=16  NOBITS

                USE32

User.Stack3.Size EQU            0010h           ; Doesn't need much at all!
User.Stack0.Size EQU            0200h           ; Needs a bit more...

User.LDT:
User.LDT.Alloc  RESB            LDT.Alloc_size
User.LDT.Alias  RESB            x86.Desc_size
User.LDT.Stack0 RESB            x86.Desc_size
User.LDT.Stack3 RESB            x86.Desc_size
User.LDT.Data   RESB            x86.Desc_size
User.LDT.Code   RESB            x86.Desc_size

%if (User.LDT.Alias-User.LDT) != DT.Alias
%error "Invalid User LDT alias"
%endif
</source>
````

<a id="w003539"></a>

## W003539 — User:Johnburger/Demo/User/Data

Bron-ID: W003539
Namespace: User (2)
Revisie: 21445 | Datum: 2017-08-29T09:32:17Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/User/Data
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FUser%2FData&oldid=21445

````text
There is not much User Data required. The [[User:Johnburger/Demo/Exec/User/TSS|Executive]] doesn't use the Data Segment to provide each Task with its unique information (mostly Window parameters); instead it sets up registers in the Task's TSS.

The first thing that the User Task does is save away those values into its own Data Segment.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/User/User|User]]/Data.inc</code> ==
<source lang="asm">
;
; User/Data.inc
;

; This module defines the Data Segment for the User Code. There will be one copy
; of this for every running program.

                  SEGMENT       User.Data  START=0  ALIGN=16  NOBITS

                  USE32

User.Data.Left    RESB          1               ; Left X co-ordinate
User.Data.Top     RESB          1               ; Top Y co-ordinate
User.Data.Width   RESB          1               ; Width of area
User.Data.Height  RESB          1               ; Height of area
User.Data.Row     RESD          1               ; Delta from one row to the next
</source>
````

<a id="w003540"></a>

## W003540 — User:Johnburger/Demo/User/Code

Bron-ID: W003540
Namespace: User (2)
Revisie: 21446 | Datum: 2017-08-29T09:32:27Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/User/Code
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FUser%2FCode&oldid=21446

````text
This code takes the starting parameters, as provided by the [[User:Johnburger/Demo/Exec/User/User|Executive]], and works out how to behave.
# Save starting parameters into Data Segment.
# Point to the Top Left of Screen area.
# Is there room to draw a frame (larger than 2x2)? If so:
## Shrink the screen area by 2 in each direction.
## Call the <code>.Frame</code> routine.
## Point inside the frame as the new Top Left corner.
# Work out the horizontal deltae, in both units (0 or 1) and screen offset (0 or 2). If the width is only 1 unit, there won't be any horizontal bouncing.
# Works out the vertical deltae, in both units (0 or 1) and screen offset (0 or Row). If the height is only 1 unit, there won't be any vertical bouncing.
# Set the start position to be 1,1. Can now do zero-checks when bouncing up or left.
# Start the loop:
## Remember the current screen pointer.
## Add the current horizontal deltae into X-Pos and screen pointer.
## Is it larger than width, or has it become zero (if negative deltae)? If so:
### Negate both deltae (change their sign/direction).
### Add deltae back into X-Pos and screen pointer.
### Twice.
## Add the current vertical deltae into Y-Pos and screen pointer.
## Is it larger than height, or has it become zero (if negative deltae)? If so:
### Negate both deltae (change their sign/direction).
### Add deltae back into Y-Pos and screen pointer.
### Twice.
## Is the new screen pointer the same as the remembered one?
### Yes: increment character at screen position to show something happened.
### No: Draw a space at the old position, and the defined ball character at the new position.
## Delay. This is one of the following techniques:
### Call <code>User.Halt</code> to perform a <code>HLT</code>. This will pause the system until the next interrupt.
### Call <code>User.Yield</code> to perform a Task Switch immediately. This will really move things along!
### Load <code>User.Delay</code> into <code>ECX</code> and perform a <code>LOOP</code>. This is busy-waiting, but to a finely-tunable time.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/User/User|User]]/Code.inc</code> ==
<source lang="asm">
;
; User/Code.inc
;

; This module defines the User code to bounce the ball. It:
; 1) Uses the initial register values to set up the Data;
; 2) Draws the Frame (assuming the defined Window is big enough);
; 3) Calculates the bounce parameters (room to move vertically or horizontally?)
; 4) Bounces the ball around (or only increments it if the Window is too small);
; Forever!
;
; Note that this code can easily write outside its Window - by using direct
; Screen access, it can poke anything anywhere. It could have been given a
; 'strip' of the Screen to access, by defining an LDT-local alias to that part
; of the screen comprising the strip. That wouldn't help accesses to the left
; or right of the Window, but it would detect other bugs!
;
; On entry:
; CS:EIP = User.Entry
; SS:ESP = Stack
; ES     = Screen
; DS     = User.Data
; EBX    = Row pointer Delta
; CL     = Width
; CH     = Height
; DL     = Left-most position
; DH     = Top-most position

                SEGMENT         User  VSTART=0  ALIGN=16

User.ColouredBall EQU           (User.BallColour << 8) | User.Ball

User.Entry:
                CLD                             ; Work forwards
                MOV             [User.Data.Row],EBX
                MOV             [User.Data.Left],DX  ; Left and Top
                MOV             [User.Data.Width],CX ; Width and Height

                PUSH            ES
                POP             DS              ; Don't need DS anymore

                MOV             EBP,EBX         ; Save this away

                ; Calculate Screen pointer
                MOV             AL,DH           ; Get Top
                MUL             BL              ; Multiply by row offset
                MOVZX           EDI,AX          ; Put into Screen ptr

                MOVZX           EAX,DL          ; Get Left
                LEA             EDI,[EDI+EAX*2] ; Add Left into Screen ptr

                ; Check if Frame is needed
                CMP             CL,2            ; Wide enough for frame?
                JBE             .DeltaX         ; Nope!
                CMP             CH,2            ; Tall enough for frame?
                JBE             .DeltaX         ; Nope!

                ; Reduce dimensions, Draw Frame, and move inside
                MOV             ESI,EDI         ; Remember current screen ptr
                SUB             CX,0202h        ; Shrink both width and height
                CALL            User.Frame      ; Draw frame...
                LEA             EDI,[ESI+EBP+2] ; ...and point inside it

.DeltaX:        ; Calculate DeltaX
                MOV             BL,+1           ; Set DeltaX
                MOV             ESI,2           ; Use this for DeltaX on pointer
                CMP             CL,1            ; Too narrow?
                JA              .DeltaY         ; Nope!
                MOV             BL,0            ; No DeltaX
                XOR             ESI,ESI         ; No DeltaX pointer

.DeltaY:        ; Calculate DeltaY
                MOV             BH,+1           ; Set DeltaY
;               MOV             EBP,EBP         ; Use this for DeltaY on pointer
                CMP             CH,1            ; Too short?
                JA              .Start          ; Nope!
                MOV             BH,0            ; No DeltaY
                XOR             EBP,EBP         ; No DeltaY pointer

.Start:
; Now that all the maths is out of the way, I need the pos to be 1-relative (not
; 0-relative)
                MOV             DH,1            ; Start Y
                MOV             DL,1            ; Start X
.Move:
                MOV             EAX,EDI         ; Remember old pointer

.MoveH:
                ADD             EDI,ESI         ; Add in DeltaX pointer
                ADD             DL,BL           ; Add in DeltaX
                JZ              .BounceH        ; Underflow?
                CMP             DL,CL           ; No. Overflow?
                JBE             .MoveV          ; No. No bounce!
.BounceH:
                NEG             ESI             ; Reverse DeltaX pointer
                NEG             BL              ; Reverse DeltaX
                LEA             EDI,[EDI+ESI*2] ; Fix up pointer
                ADD             DL,BL           ; Fix up position
                ADD             DL,BL           ; And move correctly

.MoveV:
                ADD             EDI,EBP         ; Add in DeltaY pointer
                ADD             DH,BH           ; Add in DeltaY
                JZ              .BounceV        ; Underflow?
                CMP             DH,CH           ; No. Overflow?
                JBE             .Update         ; No. No bounce!
.BounceV:
                NEG             EBP             ; Reverse DeltaY pointer
                NEG             BH              ; Reverse DeltaY
                LEA             EDI,[EDI+EBP*2] ; Fix up pointer
                ADD             DH,BH           ; Fix up position
                ADD             DH,BH           ; And move correctly

.Update:
                CMP             EDI,EAX         ; After all that, did we move?
                JE              .Increment      ; Nope! Just increment.
                MOV     BYTE    [EAX],' '       ; Erase old ball
                MOV     WORD    [EDI],User.ColouredBall ; And write in new one
                JMP             .Wait
.Increment:
                INC     BYTE    [EDI]           ; Increment screen position

.Wait:
;               HLT                             ; Illegal User-mode instruction!
                User.Halt                       ; So, use system-provided one
;                User.Yield                      ; This has a different effect

; This is an alternative to using HLT
;                PUSH            ECX
;                MOV             ECX,User.Delay  ; Wait for a little while
;                LOOP            $               ; About this long...
;                POP             ECX

                JMP             .Move
</source>
````

<a id="w003541"></a>

## W003541 — User:Johnburger/Demo/User/Frame

Bron-ID: W003541
Namespace: User (2)
Revisie: 21447 | Datum: 2017-08-29T09:32:37Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/User/Frame
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FUser%2FFrame&oldid=21447

````text
Drawing a frame is easy. I used the high OEM character set of the standard PC characters to draw the various corners and edges, and simply looped through the height and repeated widths.

The characters quoted below may not look like frame characters, but that's because they're in Unicode. The OEM equivalents come out just fine...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/User/User|User]]/Frame.inc</code> ==
<source lang="asm">
;
; User/Frame.inc
;

; This function draws the frame using ASCII art and OEM characters
User.Frame:
                PUSH            ECX

                MOV             AH,User.FrameColour
                MOV             EBX,ECX         ; Save this away

                XOR             ECX,ECX         ; Zero high part of ECX

                MOV             EDX,EDI         ; Save this away
                MOV             AL,'Ú'          ; TopLeft corner
                STOSW                           ; Store on screen
                MOV             AL,'Ä'          ; Top
                MOV             CL,BL           ; This many
                REP             STOSW           ; Store on screen
                MOV             AL,'¿'          ; TopRight corner
                STOSW                           ; Store on screen

                LEA             EDI,[EDX+EBP]   ; Go to start of next row
                MOV             CL,BH           ; Height
                MOVZX           EBX,BL          ; Don't need BH any more
.Side:
                MOV             EDX,EDI         ; Save this away
                MOV             AL,'³'          ; Side
                STOSW                           ; Store on screen
                LEA             EDI,[EDI+EBX*2] ; Move to right side
                STOSW                           ; Store on screen

                LEA             EDI,[EDX+EBP]   ; Go to start of next row
                LOOP            .Side

;               MOV             EDX,EDI         ; Save this away
                MOV             AL,'À'          ; Bottom left
                STOSW                           ; Store on screen
                MOV             AL,'Ä'          ; Bottom
                MOV             CL,BL           ; This many
                REP             STOSW           ; Store on screen
                MOV             AL,'Ù'          ; Bottom right
                STOSW                           ; Store on screen
;               LEA             EDI,[EDX+EBP]   ; (Last row anyway)

                POP             ECX
                RET
</source>
````

<a id="w003542"></a>

## W003542 — User:Johnburger/Demo/Ints/Debug/Debug

Bron-ID: W003542
Namespace: User (2)
Revisie: 28461 | Datum: 2023-11-15T08:34:54Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Debug/Debug
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FDebug%2FDebug&oldid=28461

````text
This module leverages the '386 debug facilities to implement a "debugger".

== Design ==
The original IBM PC only entered this interrupt as a result of an <code>INT 3</code> instruction in the code. The '386 added to this with the concept of "Hardware Debug Registers". These could be set to make an INT 3 occur on a number of different system events, such as executing an instruction at a particular address (without modifying that instruction to make it an INT 3 first) or, particularly usefully, writing to a particular memory location.

== Implementation ==
But all that this "debugger" does is show the memory contents of (readable) Segments on the screen. This can be VERY useful! Directly after carefully crafting a system table in memory, it is handy to add a <code>DEBUG</code> macro (<code>INT 3</code>) so that you can examine the memory, and confirm that everything is as you designed it. The alternative could be a triple-fault...

== Invocation ==
To invoke the debugger in code, merely call the <code>DEBUG</code> macro (which does an <code>INT 3</code>). Or, at runtime, press the <Pause/Break> key (get it?!). To show that the debugger is working, a [[User:Johnburger/Dingle|Dingle(tm)]] is active in the top right corner.

== Interaction ==
Once the memory dump is showing, you can Arrow or Page up and down, or choose other displayable Segments (Left and Right). You can also switch between showing Bytes and DWords by pressing &lt;Del&gt;.

=== Some Fun ===
For giggles, use the Right arrow to display the alias for the Screen, and then scroll down to see the Hex values for the changing Dingle[[User:Johnburger/Dingle|(tm)]]. Then scroll further to see the changes to the Screen that are showing the values for the changing Dingle[[User:Johnburger/Dingle|(tm)]]. Inception!

== Resume ==
To exit the Debugger, press <Esc>. It will clear the screen, return to the executing code, and continue where it left off - minus the borders, which are only drawn once at the beginning of the User Task.

== Recursion ==
One thing to note: while the Debugger is running, you can look at its operation by... invoking the Debugger. Press <Break> again. And again. And again. You'll have to exit the same number of times you entered. To prove it, go look at the Ring0 Stack that's active, and look at how deep it is. Press <Break> again. This will start a new Debugger at the beginning again, so you'll have to find the active Ring0 Stack again. Observe that it's deeper; so press <Break> again. And again. And... You had to do it, didn't you? Well, it's your own fault!

=== Stack Fault ===
That red lump of text has an Int of <code>0C</code>, which corresponds to a Stack fault. Sure enough, the Stack underflowed - the <code>ESP</code> value is at the limit of the stack. (If you don't see the red lump of text, press <Esc> a few times.) You cannot go back from here: you've caused a Stack Fault, and nothing can un-underflow it. All you can do is invoke the Debugger again. And again. And again. Want to know how deep you are? Check the <code>SS:ESP</code> value as you go. This time the value to watch out for (if you haven't changed the Stack Fault Stack Size) is <code>FFFFF000</code>. As you get nearer to this value, you get closer to underflowing the stack again. Quick: remember the values for <code>SS</code> and <code>Task</code>!

=== Double Fault ===
If you keep hitting <Break>, you'll underflow the Stack Fault handler stack, causing another Stack Fault. The CPU will vector through the Stack Fault IDT entry, which is a Task Gate, to attempt to switch to the defined Stack Fault TSS. Unfortunately, it's marked as Busy: with us! A Fault while trying to react to a Fault is usually a Double Fault, which has its own handler, INT 08. That also has a Task Gate, to a different TSS, so a whole new context is switched in, with its own new Stack - check the <code>SS</code> and <code>Task</code> values. The important thing to realise about a Double Fault isn't that it "remembers" it was in the middle of exception handler code when another exception happens: what triggers a Double Fault is if, while the CPU is trying to <em>start</em> the handler for an exception, another exception occurs.

=== Triple Fault ===
So, let's tempt fate and underflow this stack too. By pressing <Break> a few more times, the Stack will underflow again, causing a Stack Fault - but that TSS is busy, so it will generate a Double Fault. Unfortunately, even ''that'' TSS is busy, so the CPU starts a triple-fault - which simply shuts down the CPU. The PC (or virtualisation environment) detects this state, and reboots the CPU.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/[[User:Johnburger/Demo/Overview#Debug_Handler|Debug]]/Debug.inc</code> ==
<source lang="asm">
;
; Ints/Debug.inc
;

; This module leverages the '386 debug facilities to implement a "debugger".
;
; The original IBM PC only entered this interrupt as a result of an INT 3
; instruction. The '386 added to this with the concept of "Hardware Debug
; Registers". These could be set to make an INT 3 occur on a number of different
; system events: executing an instruction at a particular address (without
; modifying that instruction to make it an INT 3 first); or, particularly
; useful, writing to a particular memory location.
;
; But all that this "debugger" does is show the memory contents of (readable)
; Segments on the screen. This can be VERY useful! Directly after carefully
; crafting a system table in memory, it is handy to add a DEBUG (INT 3) so that
; you can examine the memory, and confirm that everything is as you designed it.
; The alternative could be a triple-fault...
;
; To invoke the debugger in code, merely call the "DEBUG" macro (which does an
; INT 3). Or, at runtime, press the Pause/Break key (by default: get it?!). To
; show that the debugger is working, a Dingle(tm) is active in the top right
; corner.
;
; Once the memory dump is showing, you can Scroll or Page up and down, or choose
; other displayable Segments (Left and Right). For giggles, display the alias
; for the Screen, and scroll to see the changing Dingle(tm). Then scroll further
; to see the changes to the Screen that are showing the Dingle(tm). Inception!
;
; To exit the Debugger, press Esc (by default). It will return and continue
; where it left off. One thing to note: while the Debugger is running, you can
; look at it by... invoking the Debugger. Press Break again. And again. And
; again. You'll have to exit the same number of times you entered. To prove it,
; go look at the Ring0 Stack that's active, and watch it get deeper, and deeper,
; and... You had to do it, didn't you? Well, it's your own fault!

; Here are the colours for the different fields of the display
Debug.Colour.Addr   EQU         Dev.VGA.Grey  ; Colour of Address field
Debug.Colour.Bytes  EQU         Dev.VGA.White ; Colour of Hex Bytes field
Debug.Colour.DWords EQU         Dev.VGA.White ; Colour of Hex DWords field
Debug.Colour.ASCII  EQU         Dev.VGA.Grey  ; Colour of ASCII field
Debug.Colour.Label  EQU         Dev.VGA.White ; Colour of Labels
Debug.Colour.Regs   EQU         Dev.VGA.Grey  ; Colour of Regs field
Debug.Colour.Blank  EQU         Dev.VGA.Grey  ; Colour when there's nothing there

; Here are the scan-codes for the different functions that the debugger can do
Debug.Key.Quit   EQU            Dev.Key.Break | Dev.Key.Esc ; Quit on Esc UP
Debug.Key.Up     EQU            Dev.Key.Up    ; Scroll up (earlier in memory)
Debug.Key.Down   EQU            Dev.Key.Down  ; Scroll down (later in memory)
Debug.Key.PgUp   EQU            Dev.Key.PgUp  ; Lots of .Ups
Debug.Key.PgDn   EQU            Dev.Key.PgDn  ; Lots of .Downs
Debug.Key.Format EQU            Dev.Key.Del   ; Toggle between Bytes and DWords
Debug.Key.Prev   EQU            Dev.Key.Left  ; Previous Segment
Debug.Key.Next   EQU            Dev.Key.Right ; Next Segment

; Screen coordinates to use
Debug.VGA.Top    EQU            (VGA.Rows-Debug.VGA.Height+1)/2
Debug.VGA.Left   EQU            (VGA.Cols-Debug.VGA.Width+1)/2
Debug.VGA.Width  EQU            Debug.Width.Addr+Debug.Width.Byte
Debug.VGA.Height EQU            VGA.Rows-1
Debug.Show.Height EQU           Debug.VGA.Height-Debug.Regs.Rows
Debug.Regs.Rows  EQU            4

; These constants could be hard to change...
Debug.Width.Addr  EQU           4+1+8+3
Debug.Width.Byte  EQU           (2+1)*Debug.BytesPerRow+Debug.BytesPerRow ; Chars/Byte + ASCII
Debug.Width.DWord EQU           (8+2)*Debug.BytesPerRow/4  ; Chars/DWord
Debug.BytesPerRow EQU           16

;-------------------------------------------------------------------------------
Ints.Debug:
                PUSH            0               ; Duplicate Fault's stack
                PUSH            3               ; Duplicate Fault's stack

                PUSHAD
                CALL            Ints.Fault.Frame.Create

                CALL            .Init           ; Initialise everything
                PUSH            EAX             ; Remember TSS to switch back

                CALL            .Regs           ; Display registers

                XOR             AX,AX           ; Starting Descriptor
                MOV             DS,AX
                CALL            .Desc.Next      ; Get first (real) Descriptor to Show
.Refresh:
                INC     BYTE    [ES:(VGA.Cols-1)*2] ; Display Dingle(tm)

                CALL            .Show           ; Display memory
                CALL            .Key            ; Look at keys
                CMP             AL,Debug.Key.Quit ; Finished?
                JNZ             .Refresh        ; Not yet, so re-display
.Finish:
                POP             EBX             ; Get TSS back
                CALL            .Done

                CALL            Ints.Fault.Frame.Destroy
                POPAD
                ADD             ESP,8           ; Ignore Int and "Error" values
                IRETD           ; Return to next instruction
;-------------------------------------------------------------------------------
%include        "Ints/Debug/Init.inc"   ; Initialisation functions
;-------------------------------------------------------------------------------
%include        "Ints/Debug/Regs.inc"   ; Display registers functions
;-------------------------------------------------------------------------------
%include        "Ints/Debug/Show.inc"   ; Display memory functions
;-------------------------------------------------------------------------------
%include        "Ints/Debug/Key.inc"    ; Key functions
</source>
````

<a id="w003543"></a>

## W003543 — User:Johnburger/Demo/Ints/Debug/Init

Bron-ID: W003543
Namespace: User (2)
Revisie: 21448 | Datum: 2017-08-29T09:32:51Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Debug/Init
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FDebug%2FInit&oldid=21448

````text
To initialise the debugger just requires some initial parameters to be set up, and the current TSS to be marked as a <code>[[User:Johnburger/Demo/Ints/Util|System.TSS]]</code> to prevent the Task Switcher switching away and bouncing balls all through the display.

Once the Debugger is finished, the <code>.Done</code> routine is called to clear the screen, and repair the current TSS.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/[[User:Johnburger/Demo/Ints/Debug/Debug|Debug]]/Init.inc</code> ==
<source lang="asm">
;
; Ints/Debug/Init.inc
;

; This module handles the Debug interrupt Init and Done routines.

;...............................................................................
Ints.Debug.Init:
                MOV             AX,Selector(GDT.VGA, GDT, RPL0)
                MOV             ES,AX           ; Point to Screen, for STOSW
                MOV             AX,Selector(GDT.Data, GDT, RPL0)
                MOV             FS,AX           ; Point to global (Key) Data

                ; Default Show.Row format
                CALL            Ints.Debug.Key.Format.Bytes
                CALL            Ints.System.Init
                RET

;...............................................................................
Ints.Debug.Done:
                CLD                             ; Work forwards
                MOV             AH,Debug.Colour.Blank
                MOV             AL,' '
                MOV             DH,Debug.VGA.Height
                MOV             EDI,(Debug.VGA.Top*VGA.Cols+Debug.VGA.Left)*2
.Loop:
                MOV             ECX,Debug.VGA.Width
                REP             STOSW
                ADD             EDI,(VGA.Cols-Debug.VGA.Width)*2
                DEC             DH
                JNZ             .Loop

                CALL            Ints.System.Done
                RET
</source>
````

<a id="w003544"></a>

## W003544 — User:Johnburger/Demo/Ints/Debug/Regs

Bron-ID: W003544
Namespace: User (2)
Revisie: 21449 | Datum: 2017-08-29T09:33:02Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Debug/Regs
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FDebug%2FRegs&oldid=21449

````text
== Stack Frame ==
Displaying the registers can be very useful when debugging. The only problem is getting hold of them! Luckily the default Fault handler already knows where they are - as long as this Debug handler institutes the same Stack Frame as the Fault handler.

== Register Display ==
That didn't turn out to be too hard. The next problem was where to display the registers on the screen. The Fault handler just listed them vertically. I wanted most of the screen real estate to be taken up in memory dump, so it had to be in a few rows at the bottom of the screen. I was able to encode an (X,Y) location for each register in a single byte: a screen 'map' is reproduced below from the <code>[[User:Johnburger/Demo/Ints/Util|Ints.Name]]</code> table.

Note that I cheated a little bit: I overlapped the <code>CS</code> and <code>SS</code> values over the labels for <code>EIP</code> and <code>ESP</code> respectively. I was running out of room... and besides, they just kind of work, don't they?

<source lang="asm">
;    0000000000000000111111111111111122222222222222223333333333333333
;    0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF
;000:  _EAX:########  _EBX:########  _ECX:########  _EDX:########
;040:  _ESI:########  _EDI:########  _EBP:########  Flag:########
;080:  __DS:####  __ES:####  __FS:####  __GS:####   PDBR:########
;0C0:  __CS:####:######## __SS:####:######### _LDT:#### Task:####
</source>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/[[User:Johnburger/Demo/Ints/Debug/Debug|Debug]]/Regs.inc</code> ==
<source lang="asm">
;
; Ints/Debug/Regs.inc
;

; This module displays the registers stored on the stack, with the aid of
; .Names.Regs in Debug.inc.

Ints.Debug.Regs:

.Window         EQU             ((Debug.VGA.Top+Debug.Show.Height)*VGA.Cols+Debug.VGA.Left) * 2

                PUSH            CS              ; Point to Ints.Names
                POP             DS              ; With DS

                CLD
                MOV             ESI,Ints.Names.Regs ; Register Names only
.Regs:
                LODSB                           ; Get Debug offset
                MOVZX           ECX,AL          ; Into two registers
                MOV             EDX,ECX
                AND             ECX,3Fh         ; XLoc
                SHR             EDX,6           ; YLoc
                IMUL            EDX,VGA.Cols    ; RowLoc
                ADD             EDX,ECX         ; Combined

                MOV             AH,Debug.Colour.Label
                MOV             CL,Ints.Names.Label
                LEA             EDI,[.Window+EDX*2] ; Screen position
.Name:
                LODSB                           ; Get Name
                STOSW
                LOOP            .Name
                MOV             AL,':'          ; Separator
                STOSW

                LODSB                           ; Get Hex width
                MOV             CL,AL
                LODSB                           ; Get Stack offset
                INC             ESI             ; (Ignore TSS offset)
                MOVZX           EDX,AL          ; As indexable address

                MOV             AH,Debug.Colour.Regs
                MOV             EDX,[ESP+4+EDX] ; Value to display
                CALL            Ints.Hex

                CMP             ESI,Ints.Names.End
                JB              .Regs

                RET
</source>
````

<a id="w003545"></a>

## W003545 — User:Johnburger/Demo/Ints/Debug/Show

Bron-ID: W003545
Namespace: User (2)
Revisie: 21450 | Datum: 2017-08-29T09:33:23Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Debug/Show
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FDebug%2FShow&oldid=21450

````text
The <code>.Show</code> routine is mostly just a lot of loading of memory addresses and writing Hex to the screen. There is the main loop, and three support functions:
* <code>.Address</code> displays the current Address (always a multiple of 10h);
* <code>.Show.Bytes</code> displays the current row in Bytes, with an ASCII display on the right hand side;
* <code>.Show.DWords</code> displays the current row in DWords.

The hard part is actually near the edge of the current Segment's limits. The rest of the code manages discovering the limits and changing the display address in response to keypresses: this code just has to not blow those limits. The rest of the code has already set up the input parameters to handle [[Expand Down]] Segments. All this code has to do is compare the current Segment memory pointer (<code>ESI</code>) against the Minimum (<code>EBX</code>) and Maximum (<code>EDX</code>) values, and display spaces if it's outside of those limits.

And the hardest part of that was the <code>.Show.DWords</code> displayer, since it usually uses <code>LODSD</code> to read the DWords from the current Data segment. Close to the limit of the Segment, though, this will cause a GPF - no part of the access can breach the Limit. So in that case it devolves to multiple Byte accesses to form as much of the DWord as it can assemble.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/[[User:Johnburger/Demo/Ints/Debug/Debug|Debug]]/Show.inc</code> ==
<source lang="asm">
;
; Ints/Debug/Show.inc
;

; This module has the various Display functions to show the current Segment.

;-------------------------------------------------------------------------------
; This function shows a screenful of memory, without violating any Segment
; limits. It displays the memory in either Byte (with ASCII) or DWord form.
; Input:  DS  = Segment to display
;         ES  = Screen Segment
;         EBX = Minimum displayable address (0000_0000h or Limit)
;         EDX = Maximum displayable address (Limit or FFFF_FFFFh)
;         ESI = Current address to display from
;         EBP = .Byte or .DWord function pointer
; Output: EAX, ECX, EDI modified

Ints.Debug.Show:
                PUSH            ESI

                MOV             ECX,Debug.Show.Height
                MOV             EDI,(Debug.VGA.Top*VGA.Cols+Debug.VGA.Left)*2
.Row:
                PUSH            ECX               ; Need this later

                CMP             ESI,EDX           ; Past Maximum?
                JA              .Row.Blank        ; Yes, so blank row
                CMP             ESI,EBX           ; At least Minimum?
                JAE             .Row.Show         ; Yes, so show row normally
                LEA             EAX,[EBX-Debug.BytesPerRow] ; Is it a partial row?
                CMP             ESI,EAX           ; Below even this?
                JA              .Row.Show         ; No, so show partial row
.Row.Blank:
                MOV             AH,Debug.Colour.Blank
                MOV             AL,' '            ; Blank
                MOV             ECX,Debug.VGA.Width   ; Entire row
                ADD             ESI,Debug.BytesPerRow ; Catching up?
                REP             STOSW
                JMP             .Row.Next         ; Next row
.Row.Show:
                PUSH            EDX               ; Need this later
                CALL            .Address
                POP             EDX

                CALL            EBP               ; Call current Row function

.Row.Next:
                ADD             EDI,(VGA.Cols-Debug.VGA.Width)*2
                POP             ECX               ; More rows?
                LOOP            .Row

                POP             ESI               ; Back to top
                RET
;...............................................................................
.Address:
                MOV             AH,Debug.Colour.Addr

                MOV             CL,4              ; Size of Selector
                MOV             DX,DS             ; Selector to display
                CALL            Ints.Hex

                MOV             AL,':'            ; Separator
                STOSW

                MOV             CL,8
                MOV             EDX,ESI           ; Address
                CALL            Ints.Hex

                MOV             AL,' '
                STOSW
                STOSW
                STOSW

                RET
;...............................................................................
; This function displays one row of .BytesPerRow Bytes (if inside Limits), plus
; an ASCII representation on the side.

Ints.Debug.Show.Bytes:
                MOV             AH,Debug.Colour.Bytes
                MOV             CL,Debug.BytesPerRow ; Number of Bytes
.Loop:
                PUSH            ECX

                MOV             AH,Debug.Colour.ASCII
                LEA             ECX,[Debug.BytesPerRow+ECX*2] ; Offset to ASCII field

                CMP             ESI,EBX           ; Less than Minimum?
                JB              .NoByte           ; Yes, so skip
                CMP             ESI,EDX           ; More than Maximum?
                JA              .NoByte           ; Yes, so skip

                LODSB                             ; Byte to display
                MOV             [ES:EDI+ECX*2],AX ; Store ASCII

                PUSH            EDX
                MOV             AH,Debug.Colour.Bytes
                MOV             CL,2              ; Number of nybbles
                MOV             DL,AL
                CALL            Ints.Hex
                POP             EDX

                MOV             AL,' '
                CMP     BYTE   [ESP],4            ; Don't show in last position
                JB              .Separator
                TEST            ESI,0003h
                JNZ             .Separator
                MOV             AL,'-'
.Separator:
                STOSW
                JMP             .Next
.NoByte:
                MOV             AL,' '
                INC             ESI               ; Catching up?
                MOV             [ES:EDI+ECX*2],AX ; Store ASCII

                MOV             AH,Debug.Colour.Bytes
                MOV             ECX,3
                REP             STOSW
.Next:
                POP             ECX
                LOOP            .Loop

                ; Now blank out any area to the right of the ASCII field
                MOV             AL,' '
                MOV             AH,Debug.Colour.ASCII
                MOV             ECX,Debug.VGA.Width-Debug.Width.Addr-Debug.Width.Byte
                ADD             EDI,Debug.BytesPerRow*2 ; Go past ASCII field
                REP             STOSW
                RET
;...............................................................................
; This function displays one row of 4 DWords (if inside Limits). There's some
; hairy code about trying to display part-words at the edge of the Limits...

Ints.Debug.Show.DWords:
                MOV             CL,Debug.BytesPerRow/4 ; Number of DWords
.Loop:
                PUSH            ECX

                CMP             ESI,EDX           ; More than Maximum?
                JA              .NoDWord          ; Yes, so skip
                CMP             ESI,EBX           ; Less than Minimum?
                JAE             .TestHi           ; No, so TestHi
                LEA             EAX,[EBX-4]       ; Check how FAR less we are
                CMP             ESI,EAX           ; MUCH less?
                JBE             .NoDWord          ; Yes
                JMP             .Partial          ; Get part-DWord
.TestHi:
                LEA             EAX,[EDX-3]       ; Test how close to Maximum
                CMP             ESI,EAX
                JBE             .Normal           ; Safe to proceed normally
.Partial:
                XOR             EAX,EAX           ; Start with zero
                MOV             CL,4              ; Number of bytes
.Build:
                SHL             EAX,8             ; One more byte
                CMP             ESI,EBX           ; Still too low?
                JB              .Continue         ; Yep!
                CMP             ESI,EDX           ; Now too high?
                JA              .Continue         ; Yep!
                MOV             AL,[ESI]          ; Nope, so safe!
.Continue:
                INC             ESI               ; Keep looking
                LOOP            .Build            ; Build DWord
                JMP             .Show
.Normal:
                LODSD                             ; DWord to display
.Show:
                PUSH            EDX
                MOV             CL,8              ; Number of nybbles
                MOV             EDX,EAX
                MOV             AH,Debug.Colour.DWords
                CALL            Ints.Hex
                POP             EDX
                JMP             .Separator
.NoDWord:
                MOV             AH,Debug.Colour.DWords
                MOV             AL,' '
                MOV             ECX,8
                ADD             ESI,4             ; Catching up?
                REP             STOSW
.Separator:
                MOV             AL,' '
                STOSW
                STOSW
.Next:
                POP             ECX
                LOOP            .Loop

                ; Now blank out any area to the right of the Hex field
                MOV             AL,' '
                MOV             ECX,Debug.VGA.Width-Debug.Width.Addr-Debug.Width.DWord
                REP             STOSW
                RET
</source>
````

<a id="w003546"></a>

## W003546 — User:Johnburger/Demo/Ints/Debug/Key

Bron-ID: W003546
Namespace: User (2)
Revisie: 21451 | Datum: 2017-08-29T09:33:35Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Ints/Debug/Key
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FInts%2FDebug%2FKey&oldid=21451

````text
The keyboard handler just has to test against the different keypresses that we want to heed, to affect the current display on the screen. From easiest to hardest:
* <Esc><br>Just leave the debugger;
* &lt;Del&gt;<br>Swap between displaying Bytes and DWords by changing the <code>EBP</code> register (it holds the address of the function to <code>CALL</code>).
* <Up> or <Down> arrow<br>Subtract or Add (respectively) <code>10h</code> to <code>ESI</code> (the current memory pointer) '''''unless''''' it's already outside the current Segment's limits.
* <PgUp> or <PgDn><br> Call the <Up> or <Down> (respectively) function <code>Debug.Show.Height - 1</code> times. Why <code>- 1</code>? It allows a measure of overlap for the user to maintain synchronization with the scrolling. Note that there's no attempt to break out of the loop if the Segment limits have been reached: the code simply ignores the attempt, so it can ignore it repeatedly!
* <Left> or <Right> arrow<br>These are the most complex functions. They Subtract or Add (respectively) the size of a Descriptor to the current value of the currently-displayed Selector, and then the result checked for readability with the CPU's <code>VERR</code> (VERify Read) instruction. (That hasn't had a lot of use in your code before, I'll bet!) If the new value is a readable Segment, it becomes the new current Segment. Otherwise, we loop around doing the Subtraction or Addition again. And if the ends of the Descriptor Table are reached, the [[User:Johnburger/Demo/x86/Seg|Table Indicator]] bit inside the Selector is toggled to access the LDT or GDT, and the loop is continued.

The final keypresses above call the <code>.Check</code> routine to determine if the Selector is readable, and to toggle the TI bit if necessary. If the new Selector isn't readable, it just returns. If it is, it establishes the new Minimum and Maximum values for the Segment, taking into account if the Descriptor is marked [[Expand Down]].

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Ints/Ints|Ints]]/[[User:Johnburger/Demo/Ints/Debug/Debug|Debug]]/Key.inc</code> ==
<source lang="asm">
;
; Ints/Debug/Key.inc
;

; This module handles the various keyboard actions.

Ints.Debug.Key:
                HLT                             ; Wait for interrupt

                MOV             AL,0            ; Consume any keypress
                XCHG            [FS:Data.Key.Code],AL ; Avoid repeats

                CMP             AL,0            ; Anything there?
                JE              .End            ; No, so leave
                CMP             AL,Debug.Key.Quit
                JE              .End            ; Quit?
                CMP             AL,Debug.Key.Format
                JE              .Format
                CMP             AL,Debug.Key.Up
                JE              .Up
                CMP             AL,Debug.Key.Down
                JE              .Down
                CMP             AL,Debug.Key.PgUp
                JE              .PgUp
                CMP             AL,Debug.Key.PgDn
                JE              .PgDn
                CMP             AL,Debug.Key.Prev
                JE              Ints.Debug.Desc.Prev
                CMP             AL,Debug.Key.Next
                JE              Ints.Debug.Desc.Next
.End:
                RET
;...............................................................................
Ints.Debug.Key.Format:
                CMP             EBP,Ints.Debug.Show.DWords
                JE              .Bytes
                MOV             EBP,Ints.Debug.Show.DWords
                JMP             .End
.Bytes:
                MOV             EBP,Ints.Debug.Show.Bytes
.End:
                RET
;...............................................................................
Ints.Debug.Key.Up:
                CMP             ESI,EBX          ; Reached Minimum?
                JBE             .End             ; Yep! Do nothing.

                SUB             ESI,Debug.BytesPerRow ; Move earlier in memory
.End:
                RET
;...............................................................................
Ints.Debug.Key.Down:
                MOV             EAX,ESI          ; Try some calculations
                ADD             EAX,Debug.Show.Height*Debug.BytesPerRow ; Add in bottom of Screen
                JO              .End             ; Overflow means too far!
                DEC             EAX              ; Need to, since Limit has -1
                CMP             EAX,EDX          ; Reached Maximum?
                JAE             .End             ; Yep, so too far

                ADD             ESI,Debug.BytesPerRow ; Move later in memory
.End:
                RET
;...............................................................................
Ints.Debug.Key.PgUp:
                MOV             CL,Debug.Show.Height-1 ; Leave one row the same
.Loop:
                CALL            Ints.Debug.Key.Up
                LOOP            .Loop
                RET
;...............................................................................
Ints.Debug.Key.PgDn:
                MOV             CL,Debug.Show.Height-1 ; Leave one row the same
.Loop:
                CALL            Ints.Debug.Key.Down
                LOOP            .Loop
                RET
;...............................................................................
Ints.Debug.Desc.Next:
                MOV             EAX,DS           ; Get current Descriptor
.Loop:
                ADD             AX,x86.Desc_size ; Go to next Descriptor
                CALL            Ints.Debug.Desc.Check
                JNE             .Loop
                RET

Ints.Debug.Desc.Prev:
                MOV             EAX,DS           ; Get current Descriptor
.Loop:
                SUB             AX,x86.Desc_size ; Go to previous Descriptor
                CALL            Ints.Debug.Desc.Check
                JNE             .Loop
                RET

Ints.Debug.Desc.Check:
; This checks the current Descriptor for readability. If not, it simply returns.
; If it is readable, it sets up the necessary registers; in particular the
; values for Minimum and Maximum.
; One step it does take is to check the result of the just-completed Next or
; Prev. If that over- or under-flowed, it swaps to the other Descriptor Table
; (GDT or LDT). Also note that since there's no easy way to determine the size
; of the current LDT, or even the GDT, it is easiest to simply check all the
; Descriptors.
                JNC             .Readable       ; Didn't overflow: keep going
                XOR             EAX,x86.Seg.TI  ; Use other Descriptor Table
.Readable:
                VERR            AX              ; Can I read it?
                JNZ             .End            ; Nope. Keep looking!

                MOV             DS,EAX          ; Found!

                LSL             EDX,EAX         ; Maximum address
                XOR             EBX,EBX         ; Minimum address
                MOV             ESI,EBX         ; Start at Minimum

                LAR             EAX,EAX         ; One more thing...
                TEST            AH,x86.Desc.Mem.Type.Code  ; Is it Code?
                JNZ             .Found                     ; OK, good
                TEST            AH,x86.Desc.Mem.Type.Stack ; No. Is it a Stack?
                JZ              .Found                     ; No! Phew!

                XCHG            EBX,EDX         ; Yes, so everything's upside down!
                INC             EBX             ; Or off-by-one
                NOT             EDX             ; Or inverted!
                ; Start at "top" of Segment, less a screenful
                MOV             ESI,0-(Debug.Show.Height*Debug.BytesPerRow) ; Screenful

                SHR             EAX,16          ; Also need to check Gran byte
                TEST            AL,x86.Desc.Mem.Gran.Big ; 32-bit?
                JNZ             .Found          ; Yes.
                MOV             AX,0FFFFh       ; No. Need to mask off high word
                AND             EDX,EAX         ; Maximum address
                AND             ESI,EAX         ; Starting address
.Found:
                XOR             EAX,EAX         ; Set Z flag - Found!
.End:
                RET
</source>
````

<a id="w003547"></a>

## W003547 — OSDev Wiki talk:Privacy policy

Bron-ID: W003547
Namespace: OSDev Wiki talk (5)
Revisie: 16074 | Datum: 2014-04-11T14:44:31Z
Bron: https://wiki.osdev.org/OSDev_Wiki_talk:Privacy_policy
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki_talk%3APrivacy_policy&oldid=16074

````text
: There are three typos on this page, but since it's an Admin/Legal page I hesitate to just modify it. The
typos are:
: "compile" should be "comply": "... required to comply with legal requests ..."
: "wazes" should be "warez": "Please, no warez, virus writing, ..."
: "peoples" should be "people's": "Do not post other people's private information."
````

<a id="w003548"></a>

## W003548 — Segment Limits

Bron-ID: W003548
Namespace: Main (0)
Revisie: 24065 | Datum: 2019-10-25T13:33:23Z
Bron: https://wiki.osdev.org/Segment_Limits
Revisie-URL: https://wiki.osdev.org/index.php?title=Segment_Limits&oldid=24065

````text

= Protected Mode =
In x86 Protected Mode, a Segment is described by two parameters, the Base address and the Limit:
* The Base address is where in the CPU-addressable space the Segment starts;
* The Limit is the last Segment-relative address that can be accessed offset from the Base.

For example, a Memory Descriptor entry in the Global Descriptor Table may describe an Expand Up Data Descriptor starting at <code>0005_0000h</code> for <code>0000_C000h</code> - in other words, 48 kiB (+1 byte) in the middle of Conventional RAM.

Given the above Descriptor assigned to the <code>DS</code> segment register, accessing a Byte at <code>[DS:0000_0000h]</code> or <code>[DS:0000_C000h]</code> will work, but <code>[DS:0000_C001h]</code> or above won't. The biggest advantage of this mechanism is that various offsets embedded in a program don't need to be "fixed up" at load time to cater for the load address of the program: since every offset is relative to a Segment's Base, the fix-ups are performed by the CPU at run time.

While the Base address is affected by things like Paging (if enabled), the Limit is simply the number of contiguous bytes after that: any access higher than the Limit will cause a General Protection Fault. Unless the Descriptor is for an [[#Expand_Down|Expand Down]] segment, in which case everything changes (see below)...

== 16-Bit Entry ==
When Intel introduced Descriptor Tables in the 80286 (with 16-bit data but a 24-bit address bus), they defined one Entry to be 8 bytes. That made it easy to use a Segment Register as a Selector, with a couple of leftover bits as a GDT/LDT selector and Privilege Level selector. Of the 8 bytes, they only needed 6: 3 for the (24-bit maximum) Base, 2 for the (16-bit maximum) Limit, and 1 for the Entry's Type (Data vs Code vs System). They specified that the last two bytes of the eight were reserved, and must be zero.

Since the Limit was 16 bits, Intel needed to decide how to interpret the edge conditions of 0 kiB and 64 kiB: would a Limit of <code>0000h</code> mean no access (what's the point of a zero-length Segment?) or full access (all 64 kiB available)? So (of course) they chose the third option: the Limit would be the last <em>accessible</em> byte. A Limit of <code>0000h</code> allowed only 1 accessible address, while <code>FFFFh</code> allowed all of them.

== 32-Bit Entry ==
When Intel introduced the 32-bit 80386 (with both 32-bit data and address buses), they had a problem. They needed one more byte to hold the new 32-bit Base, but <em>two</em> more bytes to hold the new 32-bit Limit - and they only had two bytes spare. They still wanted to be backward-compatible and run existing 80286 software unchanged, but they also wanted to take full advantage of the new 32-bit addresses.

But Intel realised that when dealing with really large Segments, a programmer wouldn't be agonizing over whether it should be 12,086,384 or 12,086,385 bytes in size. At those sorts of sizes - <em>especially</em> with the '386's new paging functionality - a programmer would be working at a higher granularity: in 4 kiB Pages rather than bytes. A Page needs 12 bits to address into, so if the Limit field was marked as Page-granular rather than byte-granular, it would only need to be (32-12=) 20 bits in size - and 16 were already defined!

So they made a compromise in the Descriptor Table Entry. They did add an extra byte for the Base, making it fully 32-bit addressable, but they turned the last available byte into a compound Limit-with-Flags record. The last four bits required for the maximum-sized Limit was the low nybble of that byte, and the high nybble was used for two flags, called Gran and Big:
* Gran is obvious - it indicates whether to use byte-granular (=0) or Page-granular (=1) calculations on the Limit. With a 20-bit Limit in byte-granular mode, it is possible to fine-tune a Segment to anything from 1 byte to 1 MiB. In Page-granular mode, you can specify anything from 4 kiB to 4 GiB in 4 kiB jumps. The 20-bit Limit is shifted 12 bits to the left, and <code>1</code>s are shifted in. A Page-granular Limit of <code>0_0000h</code> actually means <code>0000_0FFFh</code>.
* Big is less obvious - it indicates whether addresses used by implicit registers will be 16-bit (=0) or 32-bit (=1). What are implicit registers? There are only two: <code>(E)IP</code> and <code>(E)SP</code>. When used for a Code segment, instructions are fetched by either <code>IP</code> or <code>EIP</code>. When used for a Stack, values are pushed using either <code>SP</code> or <code>ESP</code>. The Big flag indicates which one.<br>Incidentally, when used in a Code Descriptor, the Big flag is also known as the Default flag. It sets the Default 16- or 32-bit mode that instructions will be interpreted by, when executed in this Segment. For example, the <code>B8h</code> op-code under a Default=0 Code Segment means <code>MOV AX,...</code> and needs to be followed by 2 bytes for the 16-bit value to load. Under a Default=1 Code Segment it means <code>MOV EAX,...</code> and needs to be followed by 4 bytes for the 32-bit value to load. The <code>OpSiz (66h)</code> and <code>AdSiz (67h)</code> instruction prefixes toggle the current Default interpretation.

The effect of the Big flag is subtle: it influences things like in Code Segments the size of the value pushed during a <code>CALL</code>, and in Data Segments the Segment wrap for Stacks - if <code>ESP</code> is <code>0000_0000h</code>, a <code>PUSH EAX</code> will make <code>ESP</code> either <code>0000_FFFCh</code> or <code>FFFF_FFFCh</code> depending on that flag - which doesn't mean much until you get to [[#Expand_Down|Expand Down]] Segments.

= Segmentation =
Segmentation has its detractors. Indeed, most Operating Systems today don't use it - or at least only the minimum vestigial references as required by the architecture. However, the OS then doesn't have the hardware security features inherent in Segmentation; it must perform address fix-ups at load time; and shared code becomes a nightmare. Segmentation solves all of this - abandoning it means that the OS has to duplicate the work.

Some examples might make things clearer.

== Security ==
Many of today's security bugs (and probably application bugs too) are as a result of invalid memory accesses. Whether that's through buffer under/overflows or random pointers, many security-related flaws involve accessing memory that weren't designed to be accessed. To (attempt to) counter these, all sorts of systems have been invented: [https://en.wikipedia.org/wiki/Address_space_layout_randomization| Address Space Layout Randomisation (ASLR)]; stack canaries/guard pages; deny execute access; et al. These are designed to limit the damage that accidental or malicious memory references can wreak - and often don't completely work.

The point is that Segmentation already protected against all of that! No memory access (even by the kernel) can go beyond the associated data: you can't overflow the Stack to modify Heap data; or write data then execute it as code. While it is still possible to do these tricky things, by default Segmentation '''''disables''''' this, whereas the flat memory model perforce by default enables it.

== Code ==
When you write code, you don't usually care (or need to know) where the code ends up in memory. In fact, the latest OSes actually [https://en.wikipedia.org/wiki/Address_space_layout_randomization|''randomise''] where code goes to prevent malicious use of that knowledge.

So at compile time the linker arbitrarily allocates memory locations as needed, and writes a table for the OS Loader to reference when it loads the code into memory. The main program usually gets loaded at the same place in the Virtual Address Space - but (dynamically linked) libraries may not be. If two libraries are linked with overlapping addresses, then the OS Loader has to perform a "fix-up" at load time, adding a delta to each and every embedded code reference to accommodate the change in load location. This slows down the application launch process with housekeeping. Worse, that also means that you can't "page out" the code when RAM gets tight without either writing that code to the Page File, or re-fixing it up when loading it back again.

How does Segmentation fix this? The code can refer to relative Segment locations rather than absolute memory locations. You want to jump to a routine? Reference the Segment number and the offset within that Segment. The (local) Segment can be fixed at link time - all the relative references don't change, regardless of where the code is loaded. The OS Loader can place the code at whatever absolute address it likes - it just fills in the appropriate Local Descriptor Table entry with the address - the actual code doesn't have to change. Need to page out the code and reload it? Use whatever new absolute address you like, since the Segment address doesn't change - no fix-ups required.

== Heap ==
Imagine that you define a Heap to be 4 kiB in size. You put it in its own Segment, and allow the program to <code>alloc()</code> and <code>free()</code> chunks of memory from it. If an <code>alloc()</code> comes through for more memory than is available, the code has two choices:
* It can deny the request;
* It can increase the Segment size, which may require moving the whole Segment to a new area of RAM.

This second option is where the beauty of Segmentation comes in: after the move, the Segment's Base address can be modified to point to the new memory '''''and the calling program doesn't know anything happened'''''. It still accesses the Heap with the same Selector value, and unless it uses the <code>LSL</code> (Load Segment Limit) instruction, it won't even know that the Segment grew.

== Stack ==
The point is that growing a Heap doesn't affect any of the offsets in the original data: even the stored Selector value doesn't change. Unfortunately, the same is not true for Stacks.

Now imagine that you define a 4 kiB Stack. Again it is in its own Segment, and <code>SP</code> starts at <code>1000h</code> and works its way downwards. When it hits <code>0000h</code> it's a valid address - but one more <code>PUSH</code> will make <code>SP</code> overflow the Limit. At this point the code has two choices:
* It can abort the program due to lack of Stack space;
* It can increase the Stack's size, which again may require moving the whole Segment to a new area of RAM.

The move is again not a problem: change the Segment's Base and it's done. But the Limit is the problem: there's nowhere for the Stack to grow to! Just increasing the Limit won't do anything: the Stack is at the bottom of the Segment, not the top. And moving all the data inside the Stack to the top of the new Segment <em>really</em> won't work: it will muck up all the saved offsets, since (for example) <code>1000h</code> needs to become <code>2000h</code> - but which values need to be changed? And what about the values stored in the program's registers? Which of those should be changed?

= Expand Down =
So in short, growing a Stack like you can a Heap (described [[#Heap|above]]) can't be done. At least, not using traditional Data Segments anyway, which is why Intel designed Expand Down Segments. As it sounds, they're designed to be used with Stacks to permit them to be expanded during run-time. Just by setting a single bit in the Descriptor Table Entry's Type field changes the interpretation of the whole Data Segment completely.

There are two ways to look at Expand Down Segments:
* Valid vs Invalid:
** An Expand Up Segment uses the Limit to define valid and invalid addresses above Base;
** The Expand Down flag swaps them, making the once-valid addresses invalid, and vice versa.
* Arithmetically:
** An Expand Up Segment defines the Base and the largest possible offset you're allowed to add to that Base:
** The Expand Down flag for a 32-bit Segment makes the Base actually the <em>top</em> of the accessed memory, and the Limit (+1) becomes the lowest possible offset you can access <em>below</em> that.<br>This is slightly incorrect since the CPU is still adding the offset to the Base - but since the offset is so large it's effectively negative, resulting in a <em>smaller</em> final address.
** The Expand Down flag for a 16-bit Segment has the Base 64 kiB <em>below</em> the highest accessible address.

== 16-Bit Example ==
So a 16-bit Stack of 1 kiB with a Base address of <code>05_0000h</code> and a Limit of <code>FC00h</code> (marked Expand Down, of course) would <em>start</em> with an <code>SP</code> of <code>0000h</code> - which may look worrying to some programmers. But it's perfectly safe: the first <code>PUSH AX</code> or <code>CALL</code> would make <code>SP</code> <code>FFFEh</code> and store the value there, which would actually store the value at RAM location <code>05_FFFEh</code>. Subsequent Stack operations would continue to decrease <code>SP</code> until it hit the value <code>FC00h</code> - where an access would cause a Stack Fault.

The sharp-eyed among you will have seen the difference: an Expand Up Segment's Limit is inclusive - the specified Limit is still an accessible byte. That means that in an Expand Down Segment, the Limit is <em>ex</em>clusive - it is effectively the highest address that <em>cannot</em> be accessed. So a better Limit for the above example would be <code>FBFFh</code>.

And now "growing" the Stack is easy: decrease the Limit as desired, and move the memory if necessary (not forgetting that Base is 64 kiB below the top of the area of memory that contains the data). Again, none of the offsets inside the Stack need to be modified.

== 32-Bit Example ==
A 32-bit Expand Down Segment follows on from the 16-bit version exactly, but now the 64 kiB mathematics becomes 4 GiB mathematics - which is the size of the address space! Thus, the Base can be seen as the <em>top</em> of the accessed memory, and the Limit is the highest <em>in</em>accessible location <em>below</em> that. The problem comes from the fact that the Limit only has 20 bits of resolution, and the Gran flag can't help us now.

The [[#16-Bit_Example|16-bit example]] above specified a 1 kiB Stack with a starting Limit of <code>FC00h</code> (later adjusted to <code>FBFFh</code> as explained). The 32-bit equivalent would be <code>FFFF_FBFF</code> - but that won't fit inside the 20-bit Limit field! And specifying it as Page-granular with the Gran flag won't help either - 20 bits means either <code>F_FFFFh</code> or <code>F_FFFEh</code>, which when shifted would become <code>FFFF_FFFFh</code> and <code>FFFF_EFFFh</code> respectively.

I find everything to do with 32-bit (Big) Expand Down Segments humorous:
* A Page-granular Limit of <code>F_FFFFh</code> defines a Segment whose highest <em>in</em>accessible address is <code>FFFF_FFFFh</code> - in other words, nothing is accessible!
* A byte-granular Limit of <code>0_0000h</code> defines a Segment whose highest <em>in</em>accessible address is <code>0000_0000h</code> - in other words, the lowest byte is not accessible!
* A byte-granular Limit of <code>0_FFFFh</code> defines a Segment whose lowest accessible address is 64 kiB - and everything above!

In other words, you can specify the size of a Stack to the <em>byte</em> - as long as you want it somewhere between 3.999 and 4.0 GiB in size. If you only want a 6 kiB Stack though, you either have to:
* Use the Gran bit, rounding off the size to 4 kiB chunks (4 or 8 kiB instead of 6); or
* Use a 16-bit entry (Big=0, using <code>SP</code> rather then <code>ESP</code>) with a maximum Limit of only 64 kiB. Sizes between 64 kiB and 1 MiB that can be specified to the <em>byte</em> in Expand Up Segments aren't available with Expand Down ones.

= Related Articles =
* [[Segmentation]]
* [[GDT]]

[[Category:Memory management]]
````

<a id="w003549"></a>

## W003549 — Expand Down

Bron-ID: W003549
Namespace: Main (0)
Revisie: 16091 | Datum: 2014-04-13T05:16:20Z
Bron: https://wiki.osdev.org/Expand_Down
Revisie-URL: https://wiki.osdev.org/index.php?title=Expand_Down&oldid=16091
Doorverwijzing: Segment Limits

````text
#REDIRECT [[Segment_Limits#Expand_Down]]
````

<a id="w003550"></a>

## W003550 — Pancakes:IPC EntryBased

Bron-ID: W003550
Namespace: Main (0)
Revisie: 16114 | Datum: 2014-04-13T23:45:27Z
Bron: https://wiki.osdev.org/Pancakes:IPC_EntryBased
Revisie-URL: https://wiki.osdev.org/index.php?title=Pancakes%3AIPC_EntryBased&oldid=16114

````text
== IPC Shared Memory Entry Based Ring Buffer ==
This is a lock-free implementation of an entry based ring buffer. It supports message sizes up to 16KB using a 16-bit header, but you can
easily change it to support a 24-bit or 32-bit header. Only one reader
and one writer are supported. 

I have tested this code fairly well, and there appears to be no bugs. To get a major increase in speed you can return a pointer to the inside of the buffer for reads but that is not secure unless you trust the writer. 

This is designed to be used for inter-process communication as if you have threads sharing the same address space you might find a faster method would be to have a ring buffer of static pointers and use those to point to memory and use flags to determine if the memory should be freed after the reader uses it because this would eliminate the need for the writer to copy data into the buffer.

==== Example Usage ====
<source lang="c">
RBM      rbm;
char     buf[128];
uintptr  sz;

rbm.sz = 2048 - sizeof(RB);
rbm.rb = (RB*)malloc(2048);

rb_write_nbio(&rbm, &buf[0], 128);
sz = 128;
rb_read_nbio(&rbm, &buf[0], &sz, 0);
</source>
The code above will write to the ring buffer, and then read from it. You
can have two separate threads with one reading and one writing. For bi-directional communication just use two different buffers.

==== Code ====
<source lang="c">
#ifdef B64
typedef unsigned long long  uintptr;
#else
typedef unsigned int		uintptr;
#endif
typedef unsigned long long  uint64;
typedef unsigned int		uint32;
typedef unsigned char		uint8;
typedef unsigned short		uint16;
typedef int					int32;
#endif

struct _RBE {
	int32			w;
	int32			r;
	uint8			d[];
};
typedef struct _RBE RB;

struct _RBME {
	RB				*rb;
	int32			sz;
};
typedef struct _RBME RBM;

int rb_write_nbio(RBM *rbm, void *p, uint32 sz) {
	RB volatile		*rb;
	int32			r;
	int32			w;
	uint32			*h;
	int32			x, y;
	int32			asz;
	int32			max;
	
	rb = (RB volatile*)rbm->rb;
	
	sz = sz & 0xffff;
	
	r = rb->r;
	w = rb->w;
	
	if (r > rbm->sz) {
		return 0;
	}
	
	if (w > rbm->sz) {
		return 0;
	}

	/* calculate total size including 16-bit length header */
	asz = sz + 2 + 2;
	
	/* not enough space */
	if ((w < r) && (w + asz) >= r) {
		return 0;
	}
	
	/* not enough space */
	if ((w >= r) && ((rbm->sz - w) + r) < asz) {
		return 0;
	}
	
	/* write length */
	rb->d[w++] = sz >> 8;
	if (w >= rbm->sz) {
		w = 0;
	}
	rb->d[w++] = sz & 0xff;
	if (w >= rbm->sz){
		w = 0;
	}
	
	/* split write */
	max = rbm->sz - w;
	if (w >= r && max < sz) {
		/* copy first part */
		for (x = 0; x < max; ++x) {
			rb->d[w + x] = ((uint8*)p)[x];
		}
		/* copy last part */
		for (y = 0; x < sz; ++x, ++y) {
			rb->d[y] = ((uint8*)p)[x];
		}
		
		rb->w = (w + sz) - rbm->sz;
		return 1;
	}
	
	/* straight write */
	for (x = 0; x < sz; ++x) {
		rb->d[w + x] = ((uint8*)p)[x];
	}
	
	/* 
		split read wont leave 'w' == rbm->sz but this will so we have
		to check for it and correct it else it messed up the reader 
		getting them off-track and essentially making communications
		hard to reliably recover if not impossible
	*/
	if (w + sz == rbm->sz) {
		rb->w = 0;
	} else {
		rb->w = w + sz;
	}
	return 1;
}

int rb_read_nbio(RBM *rbm, void *p, uint32 *sz, uint32 *advance) {
	RB volatile		*rb;
	int32			r;
	int32			w;
	int32			h;
	int32  			x, y;
	uint8			*_p;
	int32			max;
	
	_p = (uint8*)p;
		
	rb = (RB volatile*)rbm->rb;
	
	r = rb->r;
	w = rb->w;
	
	if (advance) {
		r = *advance;
	} else {
		if (r > rbm->sz) {
			/* bad header */
			return 0;
		}
	}
	
	if (w > rbm->sz) {
		/* bad header */
		return 0;
	}
	
	if (w == r) {
		return 0;
	}
	
	/* read size (tricky) */
	h = rb->d[r++] << 8;
	if (r == w) {
		return 0;
	}
	if (r >= rbm->sz) {
		r = 0;
	}
	h |= rb->d[r++];
	if (r == w) {
		return 0;
	}
	if (r >= rbm->sz) {
		r = 0;
	}
	
	if (h > (rbm->sz - r) + r) {
		return -1;
	}
	
	if (h > *sz) {
		return -1;
	}
	
	*sz = h;

	/* split read */
	if (r + h >= rbm->sz) {
		max = rbm->sz - r;
		for (x = 0; x < max; ++x) {
			*(_p++) = rb->d[r + x];
		}
		
		max = h - (rbm->sz - r);
		for (x = 0; x < max; ++x) {
			*(_p++) = rb->d[x];
		}
		if (advance) {
			*advance = (r + h) - rbm->sz;
		} else {
			rb->r = (r + h) - rbm->sz;
		}
		return 1;
	}
	
	/* straight read */
	for (x = 0; x < h; ++x) {
		*(_p++) = rb->d[r + x];
	}
	
	if (advance) {
		*advance = r + h;
	} else {
		rb->r = r + h;
	}
	
	return 1;
}
</source>
````

<a id="w003551"></a>

## W003551 — User:Johnburger/Demo/Exec/Exec

Bron-ID: W003551
Namespace: User (2)
Revisie: 21363 | Datum: 2017-08-28T21:13:42Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Exec
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FExec&oldid=21363

````text
The Executive is what this whole Demonstrator is about. It's the program that creates the User-mode Tasks and sets them running in their own individual environments. Of course, it is up to the Executive to create those Environments!

But first, it needs to finalise the switch to Protected Mode.

It's nice to be able to use normal coding techniques again: the severely constrained environment of the Boot sector is behind me, so I can use the standard <code>CALL</code>/<code>RET</code> paradigm again.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Executive|Exec]]/Exec.inc</code> ==
<source lang="asm">
;
; Exec/Exec.inc
;

; This module finishes the transition into Protected Mode, sets up the system to
; be less "bare bones" than was achieved via the bootstrap process, and finally
; starts to actually create some User-mode programs.
;
; First, you'll note I'm no longer using the "stream of code" paradigm that I
; used for the bootstrap - the need for that has gone away. I now have as much
; memory as I need! (Well, almost: there's still a loader limit...)
;
; Second, you'll note that there are Allocate functions to handle common
; operations, like allocating more Memory, or an entry in the GDT or an LDT.

                SEGMENT          Exec  VSTART=0  ALIGN=16

                USE32

; Phew! Made it! We're in Protected Mode! (You can un-cross your fingers now...)
; Before we can do anything else though, we need to set up a Protected Mode stack
Exec.Code:
                ; Point to Stack
                MOV             AX,Selector(Exec.LDT.Stack, LDT, RPL0)
                MOV             SS,AX
                XOR             ESP,ESP ; One of the advantages of Expand-Down Stacks!

                ; Initialise the rest of the registers
                CALL            Exec.Init

                ; Create more robust critical Fault handlers, with their own contexts
                ; To demonstrate the need for this, uncomment the next line...
;               MOV             ESP,1   ; The next stack operation will triple-fault
                CALL            Exec.Ints
                ; Once they've been installed, the same instruction is handled
;               MOV             ESP,1   ; This will fault, but at least it's handled

                ; We can now start to initialise Interrupts. First, the PICs
                CALL            Exec.PICs

                ; OK, we're all ready! Time to start the system!
                STI                             ; Enable interrupts

                ; Set up the Timer interrupt handler, and enable it
                CALL            Exec.Timer

                ; Human interaction (via the keyboard) would be nice...
                CALL            Exec.Key

                ; Everything's ready! We can start to create User-mode Programs
                JMP             Exec.User
;-------------------------------------------------------------------------------
%include        "Exec/Init.inc"         ; Initialise rest of Protected Mode
;-------------------------------------------------------------------------------
%include        "Exec/Ints/Ints.inc"    ; Install more sophisticated Fault handlers
;-------------------------------------------------------------------------------
%include        "Exec/PICs.inc"         ; Initialise PICs
;-------------------------------------------------------------------------------
%include        "Exec/Timer.inc"        ; Install Timer interrupt handler
;-------------------------------------------------------------------------------
%include        "Exec/Key.inc"          ; Install keyboard handler
;-------------------------------------------------------------------------------
%include        "Exec/User/User.inc"    ; Start creating User programs
;-------------------------------------------------------------------------------
%include        "Exec/Alloc/Alloc.inc"  ; Helper functions to allocate things
;-------------------------------------------------------------------------------
%include        "Exec/LDT.inc"          ; The Executive's LDT
;-------------------------------------------------------------------------------
%include        "Exec/GDT.inc"          ; The System's GDT
</source>
````

<a id="w003552"></a>

## W003552 — User:Johnburger/Demo/Exec/Init

Bron-ID: W003552
Namespace: User (2)
Revisie: 21435 | Datum: 2017-08-29T09:29:37Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Init
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInit&oldid=21435

````text
Now that we're in Protected Mode, the various registers need to be initialized for the new context - most of these can't even be accessed while in Real Mode!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/Init.inc</code> ==
<source lang="asm">
;
; Exec/Init.inc
;

; This finalises the initialisation of the CPU now that we're in Protected Mode.
; The Segment registers are loaded with PM-friendly values, and other registers
; are set up as needed.
;
; This is also the time to initialise the global Data area - at least, that part
; that wasn't set up by Boot. There was no real need to store the data in the
; Boot image - most of it is run-time generated anyway.
;
; And since this is a multi-tasking system that uses TSSs, the Task Register
; needs to be set up, to store the current context before switching to the new
; one. That means allocating a TSS, and initialising it with sensible values.

Exec.Init:
                ; Initialise EFlags
                PUSH   DWORD    x86.EFlags.1
                POPFD

                ; Point to global Data
                MOV             AX,Selector(GDT.Data, GDT, RPL0)
                MOV             DS,AX           ; Try to keep this here

                ; Point to Screen
                MOV             AX,Selector(GDT.VGA, GDT, RPL0)
                MOV             ES,AX           ; This will be used for lots
                MOV             FS,AX           ; Try to keep this here

                ; Point to GDT. (With GS! How apt!)
                MOV             AX,Selector(GDT.Alias, GDT, RPL0)
                MOV             GS,AX           ; Try to keep this here

                MOV    DWORD    [Data.RAM],Demo.First

                MOV     BYTE    [Data.Key.Shifts],0

                ; Now allocate Exec's TSS. Use Exec's LDT!
                MOV             AX,Selector(GDT.Exec.LDT, GDT, RPL0)
                CALL            Exec.Alloc.TSS  ; Assume this will work!
                MOV             AH,0            ; Not a System TSS
                CALL            Exec.Alloc.TSS.Enable

                ; Now any Task Switch will let Exec resume later
                LTR             BX

                RET
</source>
````

<a id="w003553"></a>

## W003553 — User:Johnburger/Demo/Exec/Ints/Ints

Bron-ID: W003553
Namespace: User (2)
Revisie: 21364 | Datum: 2017-08-28T21:13:57Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/Ints
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FInts&oldid=21364

````text
Given that most of the complex work has been factored into subroutines, installing six Fault Handlers is simply a matter of <code>CALL</code>ing the necessary routines - oh, and <code>%include</code>ing them, of course!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Overview#Install_Fault_Handlers|Ints]]/Ints.inc</code> ==
<source lang="asm">
;
; Exec/Ints/Ints.inc
;

; This module creates handlers for the critical Faults. It's only once this has
; been done that the system is truly safe from the dreaded Triple Fault!
;
; The technique is "simple" - establish a full runtime environment for critical
; Faults so that they don't rely on "piggybacking" the existing environment: in
; other words, give them each their own Stacks. And TSSs. They're almost a
; Program in their own right!
;
; The decision as to which Faults are considered "critical" is straightforward:
; 1) If you're already in the middle of a Double Fault, you'd better get into a
;    stable environment to avoid the Triple Fault!
; 2) If you get a Stack fault, you can't trust the stack any more! If it was at
;    User level, a Ring0 stack is automatically switched in. But what if you're
;    already at Ring0?
; 3) If a Task Switch is attempted, but the TSS turns out to be faulty, which
;    context are you in? The old or the new TSS? Better have a third one handy!
;
; Of course having a separate context is useful, but it introduces its own
; headaches - one of the big ones being that you'd better not suffer a second
; one of these while the first is still being handled! Oh, and better make sure
; that the Task-switching code doesn't try to switch in one of these waiting
; TSSs - they are, after all, ready to run! (We just hope they never do...)

Exec.Ints:
                CALL            Exec.Ints.Trace ; Install a Single-Step (TRACE) handler
                CALL            Exec.Ints.Debug ; Install the Debugger
                CALL            Exec.Ints.NoSeg ; Install an example NoSegment handler

                CALL            Exec.Ints.Double ; Install a Double Fault handler
                CALL            Exec.Ints.BadStack ; Install a Bad Stack handler
                CALL            Exec.Ints.BadTSS ; Install an Invalid TSS handler

                RET
;...............................................................................
%include        "Exec/Ints/Trace.inc"
;...............................................................................
%include        "Exec/Ints/Debug.inc"
;...............................................................................
%include        "Exec/Ints/NoSeg.inc"
;...............................................................................
%include        "Exec/Ints/Double.inc"
;...............................................................................
%include        "Exec/Ints/BadStack.inc"
;...............................................................................
%include        "Exec/Ints/BadTSS.inc"
;...............................................................................
%include        "Exec/Ints/Fault.inc"
</source>
````

<a id="w003554"></a>

## W003554 — User:Johnburger/Demo/Exec/Ints/Trace

Bron-ID: W003554
Namespace: User (2)
Revisie: 21436 | Datum: 2017-08-29T09:29:46Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/Trace
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FTrace&oldid=21436

````text
Install the simple [[User:Johnburger/Demo/Ints/Single|Single Step]] handler, which doesn't require its own context to run.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/Trace.inc</code> ==
<source lang="asm">
;
; Exec/Ints/Trace.inc
;

; This module installs a Single-Step (Trace) handler
Exec.Ints.Trace:
                MOV             EAX,Ints.Single
                MOV             EBX,IDT.Single
                MOV             CX,Selector(GDT.Ints, GDT, RPL0)
                MOV             DL,Type.Sys(Trap, DPL0, 386)
                CALL            Exec.Alloc.IDT
                RET
</source>
````

<a id="w003555"></a>

## W003555 — User:Johnburger/Demo/Exec/Ints/Debug

Bron-ID: W003555
Namespace: User (2)
Revisie: 21437 | Datum: 2017-08-29T09:29:56Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/Debug
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FDebug&oldid=21437

````text
Install the complex [[User:Johnburger/Demo/Ints/Debug/Debug|Debug]] handler, which (luckily) doesn't require its own context to run. In fact, it's humorous to play with the Debugger display while running in the context of the running Task!

Also, this is a ''perfect'' example of the difference between Descriptor Privilege Level (DPL) and Requested Privilege Level (RPL):
* If code wants to access a Selector, it must Request a Privilege Level as part of the Selector - the lowest two bits. The CPU will check that RPL against the Current Privilege Level (CPL) - which is the lowest two bits of the current Code Segment (<code>CS</code>). If the RPL is invalid for the current CPL, that's a GPF.
* Assuming the CPL check succeeds, it then checks against the Privilege Level recorded in the Descriptor - the DPL. If ''that's'' less than the CPL, then that's also a GPF.

So what we have here is the Debugger code that resides at Ring0. To access that code requires an RPL of 0, but that implies that the User-mode Task cannot invoke the Debugger. That may be what you want - but what if it isn't? How do you let a User-mode Task call the Debugger?

That's what Trap Gates can do. The Trap Gate doesn't need the same DPL as the Selector that it calls: it can have its own DPL. Protection is maintained since it does need to use the correct RPL for the Selector it wants to call - it's assumed that the Descriptor Table Entries are correct and define what the system designer wants!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/Debug.inc</code> ==
<source lang="asm">
;
; Exec/Ints/Debug.inc
;

; This module installs a Debug handler
Exec.Ints.Debug:
                MOV             EAX,Ints.Debug
                MOV             EBX,IDT.Debug
                MOV             CX,Selector(GDT.Ints, GDT, RPL0) ; Note needs to RPL0
                MOV             DL,Type.Sys(Trap, DPL3, 386) ; Allow User-mode Programs
                CALL            Exec.Alloc.IDT
                RET
</source>
````

<a id="w003556"></a>

## W003556 — User:Johnburger/Demo/Exec/Ints/NoSeg

Bron-ID: W003556
Namespace: User (2)
Revisie: 21438 | Datum: 2017-08-29T09:30:06Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/NoSeg
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FNoSeg&oldid=21438

````text
Install the simple [[User:Johnburger/Demo/Ints/NoSeg|No Segment]] handler, which doesn't require its own context to run.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/NoSeg.inc</code> ==
<source lang="asm">
;
; Exec/Ints/NoSeg.inc
;

; This module installs an example No Segment handler.
Exec.Ints.NoSeg:
                MOV             EAX,Ints.NoSeg
                MOV             EBX,IDT.NoSeg
                MOV             CX,Selector(GDT.Ints, GDT, RPL0)
                MOV             DL,Type.Sys(Trap, DPL0, 386)
                CALL            Exec.Alloc.IDT
                RET
</source>
````

<a id="w003557"></a>

## W003557 — User:Johnburger/Demo/Exec/Ints/Fault

Bron-ID: W003557
Namespace: User (2)
Revisie: 21427 | Datum: 2017-08-29T09:22:50Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/Fault
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FFault&oldid=21427

````text
This module provides two generic routines to handle the complexities of installing and handling Faults when a complete Task context is required for them.

== <code>Exec.Ints.Fault:</code> ==
This function sets up a mini-environment inside the Executive's [[User:Johnburger/Demo/Exec/LDT|LDT]]:
# It allocates RAM for a Stack for the Handler, and adds it to the LDT;
# It allocates RAM for a TSS, and initializes it to point to the Fault Handler;
# It adds the new TSS to the GDT, and an alias to the TSS in the LDT;
# It updates the Fault's entry in the IDT to point to the new TSS with a Task Gate.

== <code>Exec.Ints.Fault.Handler:</code> ==
This function is assumed to be called by the Handler installed above:
# It checks to see if Interrupts are OK (see [[#Extant_Interrupts|below]]);
# It then references its own TSS alias, to fish out the Backlink Task field. It's a pity that Task State Segments aren't readable by Supervisor code - this is what the Alias created above is for;
# It then brute-force copies the Descriptor for the Faulting TSS from the GDT into the (reserved) Fault Alias - that copies the <code>.Base</code> and <code>.Limit</code> fields across;
# It then updates the Fault Alias to be a mere Data Descriptor, all ready for the generic Fault handler to fish out the register values for the Faulting Task;
# It then <code>CALL</code>s the generic [[User:Johnburger/Demo/Ints/Fault|Fault]] handler - a <code>FAR</code> call since it's in the Global Interrupt Code Segment, rather than this Local Code;
# Upon return, it fixes up the Stack and <code>RET</code>urns for the next Fault.

== Extant Interrupts ==
I assume that the user may want to invoke the Debugger while the registers are being shown, which requires the Keyboard - but unfortunately the Fault might have occurred during the servicing of an interrupt!

=== Scenario ===
Here's the scenario:
* The Stack is very close to its Limit.
* A Timer interrupt occurs, which <code>PUSH</code>es <code>EFlags</code>, <code>CS</code> and <code>EIP</code> onto the Stack. The Stack is now full!
* The Timer handler needs <code>EAX</code>, so <code>PUSH</code>es it - which underflows the Stack causing a Stack Fault exception.
* The CPU switches Tasks to the Stack Fault Handler, which calls this generic Handler, which calls the generic Fault handler.

But at this point the Timer interrupt is still extant: there's a new Stack ready and able to handle the interrupt, but no way of re-triggering it! That's not so bad for the Timer interrupt, but it has a higher priority inside the [[User:Johnburger/Demo/Dev/PIC|Priority Interrupt Controller]] (PIC) than the Keyboard interrupt - so no Keypresses are going to get handled either. And of course it might have been the Keyboard interrupt that caused the Stack Fault, with the same effect: no Keypresses.

=== "Solution" ===
A na&iuml;ve programmer might just emit an End-Of-Interrupt (EOI) command (or two!) to the PIC - but that's definitely not the correct strategy. The PIC can be interrogated to find out if there are any Interrupts in-service - which is what this code does.

First, it isolates whether either the Timer or Keyboard interrupts are extant - they're the first two, with the highest priority, and I'm only interested in the Keyboard for this Demonstrator anyway. If the Timer interrupt is extant, it simply does an EOI - there'll be another in a millisecond anyway - and goes back to look again. If the Keyboard interrupt is extant, it calls its handler with an <code>INT</code> instruction.

After that, the PIC should be sufficiently "satisfied" to let more Keyboard interrupts through - and we can use the Debugger!

=== Caution ===
Note that I do '''''not''''' advocate doing this in a real system. I coded it here because I wanted to be able to debug the various memory structures after a Stack Fault, and I wanted my keyboard back!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/Fault.inc</code> ==
<source lang="asm">
;
; Exec/Ints/Fault.inc
;

; This module is the generic Fault handler, for Faults that need their own
; runtime environment: Stack, TSS, etc. It presumes the following:
; 1) The LDT has three consecutive entries for the Fault Handler:
;    a) Stack - This will be created
;    b) TSS   - This will be created and set up.
;    c) Alias - This will be used by the Fault handler to alias the Faulting TSS
; 2) The .Fault function installs the Handler using the provided values.
; 3) The .Handler function will be called by the specific Fault handler.

Exec.Ints.Fault:

Exec.Ints.Fault.Stack.Size   EQU   1000h        ; Give ourselves PLENTY of room!

%push Ints.Fault ; Let's not leave these %defines just lying around...

%define         %$IDT.Entry     EBP - 4
%define         %$LDT.Entry     EBP - 8
%define         %$Handler       EBP - 12
%define         %$Base          EBP - 16
%define         %$TSS           EBP - 20

                ENTER           20, 0

                MOV             [%$IDT.Entry],EAX
                MOV             [%$LDT.Entry],EBX
                MOV             [%$Handler],EDX

                MOV             AX,Selector(Exec.LDT.Alias, LDT, RPL0)
                MOV             ES,AX         ; Point to LDT alias

                ; Allocate Stack for Fault handler
                MOV             ECX,Exec.Ints.Fault.Stack.Size
                CALL            Exec.Alloc.RAM
                TEST            EAX,EAX
                JZ              .End

                ; Store in LDT
                MOV             DL,Type.Mem(Stack, DPL0, RW)
                MOV             DH,Gran.Mem(Byte,  Big)
                CALL            Exec.Alloc.DT.Mem

                ; Allocate TSS
                SLDT            AX            ; My LDT
                CALL            Exec.Alloc.TSS
                TEST            EAX,EAX
                JZ              .End

                ; Remember its Base
                MOV             [%$Base],EAX

                MOV             EBX,[%$LDT.Entry]
                OR              BX,WORD x86.Seg.TI      ; Index inside LDT
                MOV             EDX,[%$Handler]

                MOV             [ES:x86.TSS.SS],BX
                MOV             [ES:x86.TSS.CS],CS
                MOV             [ES:x86.TSS.EIP],EDX

                MOV             AH,System.TSS   ; Oh BOY is it a System TSS!
                CALL            Exec.Alloc.TSS.Enable

                MOV             [%$TSS],EBX

                MOV             AX,Selector(Exec.LDT.Alias, LDT, RPL0)
                MOV             ES,AX             ; Point to LDT alias again

                MOV             EAX,[%$Base]
                MOV             EBX,[%$LDT.Entry]
                ADD             EBX,x86.Desc_size ; Point to .TSS alias in LDT
                MOV             DL,Type.Mem(Data, DPL0, NoRW) ; TSS Alias
                MOV             DH,Gran.Mem(Byte, Small)
                CALL            Exec.Alloc.DT.Mem

                ; Finally, update IDT Entry to tell it it's now handled by a TSS
                XOR             EAX,EAX
                MOV             EBX,[%$IDT.Entry]
                MOV             ECX,[%$TSS]
                MOV             DL,Type.Sys(Task, DPL0, 286) ; No '386 variant
                CALL            Exec.Alloc.IDT
.End:
                LEAVE
                RET
%pop
;...............................................................................
.Handler:
                STI                           ; Interrupts may have been off

.TestIRQs:
                ; Worse, Fault may have happened during an IRQ handler!
                ; Isolate IRQ handlers that have started but aren't acknowledged
                MOV             AL,Dev.PIC.Cmd.ISR
                OUT             Dev.PIC.A.Cmd,AL
                JMP             $+2             ; Wait for I/O bus
                IN              AL,Dev.PIC.A.Cmd

                AND             AL,Dev.PIC.A.Key|Dev.PIC.A.Timer; Only worried about these
                JZ              .NoInterrupts    ; Nothing to worry about!
                CMP             AL,Dev.PIC.A.Key ; If keypress outstanding...
                JE              .KeyInterrupt    ; Handle it
                MOV             AL,Dev.PIC.Cmd.EOI ; Acknowledge
                OUT             Dev.PIC.A.Cmd,AL ; current IRQ (should be Timer)
                JMP             .TestIRQs        ; And keep looking

.KeyInterrupt:
                INT             (IDT.Key-IDT)/x86.Desc_size ; Re-invoke the Key interrupt
; Continue
.NoInterrupts:
                MOV             AX,Selector(GDT.Alias, GDT, RPL0)
                MOV             DS,AX         ; Need information from GDT
                MOV             AX,Selector(Exec.LDT.Alias, LDT, RPL0)
                MOV             ES,AX         ; Need to update alias in LDT
                LEA             AX,[BX + x86.Seg.TI] ; Need my TSS alias in LDT
                MOV             FS,AX

                ADD             EBX,x86.Desc_size ; Point to TSS alias in LDT

                MOVZX           ESI,WORD [FS:x86.TSS.Back] ; Who called me?

                PUSH            ESI               ; Need this on Stack to display Task

                ; Copy faulting TSS definition, then alter .Type to make it an alias
                CLD                               ; Work forwards
                MOV             ECX,x86.Desc_size / 4      ; Number of DWords
                MOV             EDI,EBX
                REP             MOVSD             ; Brute force details across!
                MOV             [ES:EBX+x86.Desc.Type],BYTE Type.Mem(Data, DPL0, NoRW)

                LEA             AX,[BX + x86.Seg.TI] ; TSS alias in LDT
                MOV             FS,AX

                MOV             EDI,[ESP+8]       ; Load Interrupt number
                CALL            GDT.Ints : Ints.Fault.External

                ADD             ESP,4             ; Ignore Back Task
                RET
</source>
````

<a id="w003558"></a>

## W003558 — User:Johnburger/Demo/Exec/Ints/Double

Bron-ID: W003558
Namespace: User (2)
Revisie: 21439 | Datum: 2017-08-29T09:30:15Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/Double
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FDouble&oldid=21439

````text
There are two aspects of the Double Fault handler: the installer, and the handler itself. The generic routines have been [[User:Johnburger/Demo/Exec/Ints/Fault|written]], it's just a matter of calling them.

Note that when a Task does an <code>IRET</code>, it picks up again from the next instruction! I don't know about you, but it looks weird to me...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/Double.inc</code> ==
<source lang="asm">
;
; Exec/Ints/Double.inc
;

; Ths module installs a Double Fault handler. It's mostly a stub, leaving the
; hard work to the generic .Fault handler.
Exec.Ints.Double:
                MOV             EAX,IDT.Double  ; This is the IDT Entry
                MOV             EBX,Exec.LDT.Double ; This is the LDT triple
                MOV             EDX,.Handler    ; This is the handler
                CALL            Exec.Ints.Fault ; Set it up
                RET
.Handler:
;               PUSH   DWORD    0               ; Pseudo-fault code (not required)
                PUSH            8               ; Interrupt number
                MOV             EBX,Exec.LDT.Double.TSS ; For TSS Backlink
                CALL            Exec.Ints.Fault.Handler
                ADD             ESP,8           ; Get rid of above
                IRETD                           ; Return, but resumes here, so...
                JMP             .Handler        ; ...go back for more
</source>
````

<a id="w003559"></a>

## W003559 — User:Johnburger/Demo/Exec/Ints/BadStack

Bron-ID: W003559
Namespace: User (2)
Revisie: 21440 | Datum: 2017-08-29T09:30:27Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/BadStack
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FBadStack&oldid=21440

````text
There are two aspects of the Stack Fault handler: the installer, and the handler itself. The generic routines have been [[User:Johnburger/Demo/Exec/Ints/Fault|written]], it's just a matter of calling them.

Note that when a Task does an <code>IRET</code>, it picks up again from the next instruction! I don't know about you, but it looks weird to me...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/BadStack.inc</code> ==
<source lang="asm">
;
; Exec/Ints/BadStack.inc
;

; Ths module installs a Stack Fault handler. It's mostly a stub, leaving the
; hard work to the generic .Fault handler.
Exec.Ints.BadStack:
                MOV             EAX,IDT.BadStack ; This is the IDT Entry
                MOV             EBX,Exec.LDT.BadStack ; This is the LDT triple
                MOV             EDX,.Handler     ; This is the handler
                CALL            Exec.Ints.Fault  ; Set it up
                RET
.Handler:
;               PUSH   DWORD    0               ; Pseudo-fault code (not required)
                PUSH            12              ; Interrupt number
                MOV             EBX,Exec.LDT.BadStack.TSS ; For TSS Backlink
                CALL            Exec.Ints.Fault.Handler
                ADD             ESP,8           ; Get rid of above
                IRETD                           ; Return, but resumes here, so...
                JMP             .Handler        ; ...go back for more
</source>
````

<a id="w003560"></a>

## W003560 — User:Johnburger/Demo/Exec/Ints/BadTSS

Bron-ID: W003560
Namespace: User (2)
Revisie: 21441 | Datum: 2017-08-29T09:30:36Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Ints/BadTSS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FInts%2FBadTSS&oldid=21441

````text
There are two aspects of the Invalid TSS handler: the installer, and the handler itself. The generic routines have been [[User:Johnburger/Demo/Exec/Ints/Fault|written]], it's just a matter of calling them.

Note that when a Task does an <code>IRET</code>, it picks up again from the next instruction! I don't know about you, but it looks weird to me...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Ints/Ints|Ints]]/BadTSS.inc</code> ==
<source lang="asm">
;
; Exec/Ints/BadTSS.inc
;

; Ths module installs a Bad TSS Fault handler. It's mostly a stub, leaving the
; hard work to the generic .Fault handler.
Exec.Ints.BadTSS:
                MOV             EAX,IDT.BadTSS  ; This is the IDT Entry
                MOV             EBX,Exec.LDT.BadTSS ; This is the LDT triple
                MOV             EDX,.Handler    ; This is the handler
                CALL            Exec.Ints.Fault ; Set it up
                RET
.Handler:
;               PUSH   DWORD    0               ; Pseudo-fault code (not required)
                PUSH            10              ; Interrupt number
                MOV             EBX,Exec.LDT.BadTSS.TSS ; For TSS Backlink
                CALL            Exec.Ints.Fault.Handler
                ADD             ESP,8           ; Get rid of above
                IRETD                           ; Return, but resumes here, so...
                JMP             .Handler        ; ...go back for more
</source>
````

<a id="w003561"></a>

## W003561 — User:Johnburger/Demo/Exec/PICs

Bron-ID: W003561
Namespace: User (2)
Revisie: 21442 | Datum: 2017-08-29T09:30:45Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/PICs
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FPICs&oldid=21442

````text
Revectoring the hardware Interrupt ReQuests (IRQs) from their legacy position to a more sensible one is something that nearly every x86-based system should do. The code is well understood, and most everyone does it the same way - and often with the same new values of 20h and 28h too!

But this time, with symbols!

You'll note that the last thing this code does is to re-disable interrupts. I ''really'' wish that initialising the PICs would have them by default leave all interrupts disabled. Unfortunately, it does the exact opposite. Ah well: I need to enable the Cascade interrupt on the Master anyway...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/PICs.inc</code> ==
<source lang="asm">
;
; Exec/PICs.inc
;

; To service hardware Interrupt ReQuests, the PICs need to be changed from their
; arbitrary BIOS-set Real Mode values to more rational settings. Other modules
; can then install IRQ handlers for their particular devices.
;
; To program a PIC, you first give it the Init Cmd, then you program the Mask
; register a few times in a prescribed order. The details change depending on
; whether the PIC is a Master or a Slave.

Exec.PICs:
                MOV             AL,Dev.PIC.Cmd.Init     ; Initialise PICs
                OUT             Dev.PIC.A.Cmd,AL
                OUT             Dev.PIC.B.Cmd,AL

                MOV             AL,PIC.A.Base           ; Set PIC.A's Base
                OUT             Dev.PIC.A.Mask,AL
                MOV             AL,PIC.B.Base           ; Set PIC.B's Base
                OUT             Dev.PIC.B.Mask,AL

                MOV             AL,Dev.PIC.A.Cascade    ; Tell PIC.A where Slave is
                OUT             Dev.PIC.A.Mask,AL

                MOV             AL,Dev.PIC.B.Cascade    ; Tell PIC.B where it is Slave
                OUT             Dev.PIC.B.Mask,AL

                MOV             AL,Dev.PIC.Init.8086    ; 80x86 mode
                OUT             Dev.PIC.A.Mask,AL
                OUT             Dev.PIC.B.Mask,AL

                MOV             AL,~Dev.PIC.A.Cascade   ; Allow Cascade interrupts through
                OUT             Dev.PIC.A.Mask,AL
                MOV             AL,0FFh                 ; Mask all interrupts
                OUT             Dev.PIC.B.Mask,AL

                RET
</source>
````

<a id="w003562"></a>

## W003562 — User:Johnburger/Demo/Exec/Timer

Bron-ID: W003562
Namespace: User (2)
Revisie: 21405 | Datum: 2017-08-29T09:11:22Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FTimer&oldid=21405

````text
This code needs to do three things:
* Install the [[User:Johnburger/Demo/Ints/Timer|Timer interrupt handler]];
* Speed up the [[User:Johnburger/Demo/Dev/Timer|Timer]] to a more useful/humourous 1,000 times per second;
* Enable Timer interrupts in its [[User:Johnburger/Demo/Dev/PIC|PIC]].

Note that the new Timer rate is [[User:Johnburger/Demo/Demo.inc|configurable]].

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/Timer.inc</code> ==
<source lang="asm">
;
; Exec/Timer.inc
;

; This module installs a Timer interrupt handler, then sets the Programmable
; Interval Timer (PIT) to interrupt often. The Interrupt handler will cycle
; through the active Tasks, emulating simultaneity.

Exec.Timer:
; First, install a Timer interrupt handler
                MOV             EAX,Ints.Timer  ; Offset
                MOV             EBX,IDT.Timer   ; Interrupt
                MOV             CX,Selector(GDT.Ints, GDT, RPL0)
                MOV             DL,Type.Sys(Int, DPL0, 386)
                CALL            Exec.Alloc.IDT

; Then, reprogram the Programmable Interrupt Timer to the desired frequency
                MOV             AX,Dev.Timer.ClockFreq / Timer.Tick

                OUT             Dev.Timer.Port,AL ; Low byte
                JMP             $+2               ; (Wait for it...)
                MOV             AL,AH             ; High byte
                OUT             Dev.Timer.Port,AL

; Finally, enable the Timer interrupt in the PIC
                IN              AL,Dev.PIC.A.Mask
                AND             AL,~Dev.PIC.A.Timer ; Unmask Timer IRQ
                OUT             Dev.PIC.A.Mask,AL

                RET
</source>
````

<a id="w003563"></a>

## W003563 — User:Johnburger/Demo/Exec/Key

Bron-ID: W003563
Namespace: User (2)
Revisie: 21425 | Datum: 2017-08-29T09:22:12Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Key
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FKey&oldid=21425

````text
This code needs to do two things:
* Install the [[User:Johnburger/Demo/Ints/Key|Keyboard interrupt handler]];
* Enable [[User:Johnburger/Demo/Dev/Key|Keyboard]] interrupts in its [[User:Johnburger/Demo/Dev/PIC|PIC]].

Note that this Interrupt handler ''doesn't'' do two things:
# Maintain the state of the "Shift" keys, like <Shift>, <Ctrl> and <Alt>;
# Convert scan codes into ASCII for the standard keys.

Both of these are left as an exercise for the reader - including the decision as to whether they belong inside the interrupt handler at all! (Hint: I suggest "Yes" for #1 and "No" for #2).

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/Key.inc</code> ==
<source lang="asm">
;
; Exec/Key.inc
;

; This module installs a Keyboard interrupt handler. There's not much to do:
; the defaults as set by the BIOS are good enough.
; Of course, to be a true Device Driver, something needs to convert the KeyCodes
; generated by the keyboard into the ASCII equivalents (or whatever) for more
; general consumption by other programs. This has been left as an exercise for
; the reader - until I get impatient and do it myself...

Exec.Key:
; First, install a Keyboard interrupt handler
                MOV             EAX,Ints.Key    ; Handler
                MOV             EBX,IDT.Key     ; Interrupt
                MOV             CX,Selector(GDT.Ints, GDT, RPL0)
                MOV             DL,Type.Sys(Int, DPL0, 386)
                CALL            Exec.Alloc.IDT

; Finally, enable the Keyboard interrupt in the PIC
                IN              AL,Dev.PIC.A.Mask
                AND             AL,~Dev.PIC.A.Key         ; Unmask Key IRQ
                OUT             Dev.PIC.A.Mask,AL

                RET
</source>
````

<a id="w003564"></a>

## W003564 — User:Johnburger/Demo/Exec/LDT

Bron-ID: W003564
Namespace: User (2)
Revisie: 21313 | Datum: 2017-08-28T20:50:50Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/LDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FLDT&oldid=21313

````text
I wanted to define the Executive's LDT in source code, since it wouldn't be growing in size. As it turned out though, parts of it needed to be filled in at run-time anyway. The reserved entries are at the bottom.

== <code></code> ==
<source lang="asm">
;
; Exec/LDT.inc
;

; This is the Local Descriptor Table for the Executive

                SEGMENT         Exec.LDT  VSTART=0  ALIGN=16

                USE32

; This macro makes it easy to have a one-line invocation of LDT.Alloc
; The parameters are: GDT Entry, LDT Limit
%macro          LDT.Vars                2
%00             ISTRUC                  LDT.Alloc
AT LDT.Alloc.Free,   DW                 0
AT LDT.Alloc.Limit,  DW                 %2
                     DB                 0
AT LDT.Alloc.Type,   DB                 x86.Desc.Sys.Type.LDT
AT LDT.Alloc.GDT,    DW                 %1
                IEND
%endmacro

Exec.LDT.Alias.Base  EQU        Exec.LDT.Base
Exec.LDT.Alias.Limit EQU        Exec.LDT.Limit
Exec.LDT.Alias.Type  EQU        Type.Mem(Data, DPL0, RW)
Exec.LDT.Alias.Gran  EQU        Gran.Mem(Byte, Small)

Exec.LDT.IDT.Base    EQU        IDT.Base
Exec.LDT.IDT.Limit   EQU        IDT.Max - 1
Exec.LDT.IDT.Type    EQU        Type.Mem(Data, DPL0, RW)
Exec.LDT.IDT.Gran    EQU        Gran.Mem(Byte, Small)

Exec.LDT.Stack.Top   EQU        Exec.Stack.Top            ; Let's play with Expand-Down Descriptors
Exec.LDT.Stack.Limit EQU        (0-Exec.Stack.Size) >> 12 - 1
Exec.LDT.Stack.Type  EQU        Type.Mem(Stack, DPL0, RW)
Exec.LDT.Stack.Gran  EQU        Gran.Mem(Gran, Big)

Exec.LDT.Code.Base   EQU        Exec.Base
Exec.LDT.Code.Limit  EQU        Exec.Limit
Exec.LDT.Code.Type   EQU        Type.Mem(Code, DPL0, NoRW)
Exec.LDT.Code.Gran   EQU        Gran.Mem(Byte, Def32)

Exec.LDT:
Exec.LDT.Vars   LDT.Vars        GDT.Exec.LDT, Exec.LDT.Limit
Exec.LDT.Alias  Desc.Mem        Exec.LDT.Alias.Base, Exec.LDT.Alias.Limit, Exec.LDT.Alias.Type, Exec.LDT.Alias.Gran
Exec.LDT.IDT    Desc.Mem        Exec.LDT.IDT.Base,   Exec.LDT.IDT.Limit,   Exec.LDT.IDT.Type,   Exec.LDT.IDT.Gran
Exec.LDT.Stack  Desc.Mem        Exec.LDT.Stack.Top,  Exec.LDT.Stack.Limit, Exec.LDT.Stack.Type, Exec.LDT.Stack.Gran
Exec.LDT.Code   Desc.Mem        Exec.LDT.Code.Base,  Exec.LDT.Code.Limit,  Exec.LDT.Code.Type,  Exec.LDT.Code.Gran

Exec.LDT.Double         EQU       $
Exec.LDT.Double.Stack   Desc.Mem  0, 0, 0, 0 ; Stack for Double Fault handler
Exec.LDT.Double.TSS     Desc.Mem  0, 0, 0, 0 ; Alias for TSS to read Backlink
Exec.LDT.Double.Alias   Desc.Mem  0, 0, 0, 0 ; Alias for Faulting TSS

Exec.LDT.BadStack       EQU       $
Exec.LDT.BadStack.Stack Desc.Mem  0, 0, 0, 0 ; Stack for BadStack Fault handler
Exec.LDT.BadStack.TSS   Desc.Mem  0, 0, 0, 0 ; Alias for TSS to read Backlink
Exec.LDT.BadStack.Alias Desc.Mem  0, 0, 0, 0 ; Alias for Faulting TSS

Exec.LDT.BadTSS         EQU       $
Exec.LDT.BadTSS.Stack   Desc.Mem  0, 0, 0, 0 ; Stack for BadTSS Fault handler
Exec.LDT.BadTSS.TSS     Desc.Mem  0, 0, 0, 0 ; Alias for TSS to read Backlink
Exec.LDT.BadTSS.Alias   Desc.Mem  0, 0, 0, 0 ; Alias for Faulting TSS

; Note that this LDT cannot grow during run-time - there's other stuff directly after it!

%if (Exec.LDT.Alias-Exec.LDT) != DT.Alias
%error "Invalid Executive LDT alias"
%endif
</source>
````

<a id="w003565"></a>

## W003565 — User:Johnburger/Demo/Exec/GDT

Bron-ID: W003565
Namespace: User (2)
Revisie: 21424 | Datum: 2017-08-29T09:21:17Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/GDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FGDT&oldid=21424

````text
The GDT's first entry is defined to be unused, to allow a <code>NULL</code> Segment selector to always cause a GPF. That makes for handy storage for an allocator - and I re-used the logic for LDTs, even though they don't have their first entry unused (I just mark it as non-present - oh, and an LDT, which is not legal inside an LDT!)

This GDT definition has entries for:
* The Descriptor Table Allocator storage (as described above);
* The GDT Alias (another expectation of the Descriptor Table Allocator code);
* Global Data, for access by global Code such as Interrupt handlers;
* The [[User:Johnburger/Demo/Dev/VGA|VGA]] text screen;
* The global Interrupt handling Code;
* The [[User:Johnburger/Demo/Exec/Exec|Executive]]'s LDT.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/GDT.inc</code> ==
<source lang="asm">
;
; Exec/GDT.inc
;

; The GDT is the only Descriptor Table that can store certain system tables,
; like Local Descriptor Tables and TSSes. If you think about it, that's because
; at Task Switch time, the GDT is the only place those entries will be guaranteed
; to be, regardless of which Task is currently running.
; That logic also applies to Code Segments for interrupt handlers.
; And those Interrupt handlers may need access to global Data, and the Screen...
; Everything else can be stored in each Task's Local Descriptor Table.
;
; That means that the Executive has nothing: no Code to run, no Stacks to use,
; and no Data to reference. To solve this, we give the Executive its own LDT.

                SEGMENT         GDT  VSTART=0  ALIGN=16

                USE32

; This macro makes it easy to have a one-line invocation of GDT.Alloc
; The parameters are: GDT Base, GDT Limit
%macro          GDT.Vars                2
%00             ISTRUC                  GDT.Alloc
AT GDT.Alloc.Free,         DW           0
AT GDT.Alloc.Limit,        DW           %2
AT GDT.Alloc.Pseudo.Base,  DD           %1
                IEND
%endmacro

GDT.Alias.Base  EQU             GDT.Base
GDT.Alias.Limit EQU             (GDT.Size-1) >> 12     ; Let's use Granular limits
GDT.Alias.Type  EQU             Type.Mem(Data, DPL0, RW)
GDT.Alias.Gran  EQU             Gran.Mem(Gran, Small)

GDT.Data.Base   EQU             Data.Base
GDT.Data.Limit  EQU             Data.Size - 1
GDT.Data.Type   EQU             Type.Mem(Data, DPL0, RW)
GDT.Data.Gran   EQU             Gran.Mem(Byte, Big)

GDT.VGA.Base    EQU             Dev.VGA.Base
GDT.VGA.Limit   EQU             VGA.Rows * VGA.Cols * 2 - 1
GDT.VGA.Type    EQU             Type.Mem(Data, DPL3, RW) ; Let anyone access it!
GDT.VGA.Gran    EQU             Gran.Mem(Byte, Big)

GDT.Ints.Base   EQU             Ints.Base
GDT.Ints.Limit  EQU             Ints.Limit
GDT.Ints.Type   EQU             Type.Mem(Code, DPL0, RW) ; Uses lookup tables
GDT.Ints.Gran   EQU             Gran.Mem(Byte, Def32)

GDT.Exec.LDT.Base  EQU          Exec.LDT.Base
GDT.Exec.LDT.Limit EQU          Exec.LDT.Limit
GDT.Exec.LDT.Type  EQU          Type.Sys(LDT, DPL0, 286)
GDT.Exec.LDT.Gran  EQU          Gran.Mem(Byte, Small)

GDT:
GDT.NULL        GDT.Vars        GDT.Base, GDT.Limit
GDT.Alias       Desc.Mem        GDT.Alias.Base,    GDT.Alias.Limit,    GDT.Alias.Type,    GDT.Alias.Gran
GDT.Data        Desc.Mem        GDT.Data.Base,     GDT.Data.Limit,     GDT.Data.Type,     GDT.Data.Gran
GDT.VGA         Desc.Mem        GDT.VGA.Base,      GDT.VGA.Limit,      GDT.VGA.Type,      GDT.VGA.Gran
GDT.Ints        Desc.Mem        GDT.Ints.Base,     GDT.Ints.Limit,     GDT.Ints.Type,     GDT.Ints.Gran
GDT.Exec.LDT    Desc.Mem        GDT.Exec.LDT.Base, GDT.Exec.LDT.Limit, GDT.Exec.LDT.Type, GDT.Exec.LDT.Gran

GDT.Size        EQU             8192 * x86.Desc_size

%if (GDT.Alias-GDT) != DT.Alias
%error "Invalid GDT alias"
%endif
</source>
````

<a id="w003566"></a>

## W003566 — User:Johnburger/Demo/Exec/User/User

Bron-ID: W003566
Namespace: User (2)
Revisie: 21365 | Datum: 2017-08-28T21:14:12Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/User/User
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FUser%2FUser&oldid=21365

````text
For each Task we want to set up, the Main Executive Task will perform the following steps:
# Prepare the LDT (<code>[[User:Johnburger/Demo/Exec/User/LDT|Demo/Exec/User/LDT]]</code>)
## Allocate memory for an LDT, and put a descriptor for it in the GDT
## Allocate memory for a Ring0 Stack, and put a descriptor for it in the LDT
## Allocate memory for a Ring3 Stack, and put a descriptor for it in the LDT
## Allocate memory for a Data Segment, and put a descriptor for it in the LDT
## Put a descriptor for the Code Segment in the LDT (share code among all tasks!)
# Prepare the TSS (<code>[[User:Johnburger/Demo/Exec/User/TSS|Demo/Exec/User/TSS]]</code>)
## Allocate memory for a TSS, fill it in, and put a descriptor for it in the GDT

Note that the last step will make the new TSS runnable: a new Program has been created!

Each new Task needs to be assigned a different area of the screen to modify. This information is easiest passed into the Task by altering its starting registers. We could modify the Task's Data Segment instead - that is left as an exercise for the reader!

Once all the Tasks have been created, there's nothing for the Main Task to do. It might as well simply <code>HLT</code> - but an Interrupt will take it out of <code>HLT</code>, so <code>JMP</code> back and do it all again! Or, we could simply remove the Main Task's TSS from the run queue. (Run queue? Run queue!? What's that?)

Of course, this is also a perfect location to insert some test code: <code>MOV ESP,1</code> anybody?

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Overview#Create_User_Tasks|User]]/User.inc</code> ==
<source lang="asm">
;
; Exec/User/User.inc
;

; Now that everything is set up, we can start to generate User-mode Programs.
; Each Program is essentially the same: code that bounces a ball around its
; defined area of the screen. To fully demonstrate the features of Protected
; Mode, the following is used:
; 1) The Code runs in User Mode (Ring 3);
; 2) Every Program has its own LDT;
; 3) The CPU's Task Switching mechanism using TSSs is used;
; 4) Every Program has its own Stacks and Data, but shares the same Code.
; Once all the Tasks have been set up, the Executive merely waits for
; <Ctrl><Alt><Del> to be pressed - and then resets the computer... At least it
; would: that's been left as an exercise for the reader. At the moment, you can
; use the trusty Power button!

%push Exec.Start ; Let's not leave these %defines just lying around...

%define         %$Pos           EBP - 4
%define         %$LDT           EBP - 8

;...............................................................................
%include        "Exec/User/LDT.inc" ; Creates one LDT for a User-mode Program
;...............................................................................
%include        "Exec/User/TSS.inc" ; Creates one TSS for a User-mode Program

Exec.User:
                ENTER           8, 0

                XOR             EDX,EDX         ; Zero high part of EDX
;               MOV             DL, 0           ; Start full left
                MOV             DH, 1           ; Start one row low

.Loop:
                MOV             [%$Pos],EDX     ; Store away new position

                ; Create a new User LDT
                CALL            Exec.User.LDT
                TEST            EAX,EAX
                JZ              .Finished

                ; Create a TSS to run that User Task
                CALL            Exec.User.TSS
                TEST            EAX,EAX
                JZ              .Finished

                MOV             EDX,[%$Pos]              ; Current position
                ADD             DL,Window.Width          ; Next Window
                CMP             DL,VGA.Cols-Window.Width ; Too far?
                JBE             .Loop                    ; Not yet...

                MOV             DL,0            ; Start at left again
                ADD             DH,Window.Height          ; Next row
                CMP             DH,VGA.Rows-Window.Height ; Too far?
                JBE             .Loop                     ; Not yet...

.Finished:
                HLT
; Wait for <Ctrl><Alt><Del>
                JMP             .Finished

%pop
</source>
````

<a id="w003567"></a>

## W003567 — User:Johnburger/Demo/Exec/User/LDT

Bron-ID: W003567
Namespace: User (2)
Revisie: 21423 | Datum: 2017-08-29T09:20:36Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/User/LDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FUser%2FLDT&oldid=21423

````text
The Executive needs to perform the following to create a new LDT:
* First the Executive needs to allocate some memory for the new LDT - but since it will be writing to it, it needs to start as a Data Segment.
* Then, for each structure in the [[User:Johnburger/Demo/User/LDT|User's LDT]], the Executive has to:
** Allocate some memory for it (unless it's the [[User:Johnburger/Demo/User/Code|User Code]] - that's already assembled!);
** Potentially initialize its contents (don't bother for Stacks);
** Add a Descriptor to the LDT with the correct settings.
* Finally, the Data Segment can be converted into a true LDT.

I've been lazy and not called <code>[[User:Johnburger/Demo/Exec/Alloc/RAM|Demo/Exec/Alloc/RAM]]</code>'s <code>.Zero</code> routine for the allocated RAM. Not doing this is a security risk - but since I wrote the code that uses it, I promise (hand-on-heart) not to mis-use any uninitialised data I may find... Besides, to initialize it would require assigning it to a Descriptor that I can access - the LDT under construction isn't an LDT yet!

'''''CAUTION:''''' If you do decide to switch your own LDT, note that you '''must''' modify the LDT field in your TSS '''first''', before ''immediately'' changing it with an <code>LLDT</code> instruction. This is because the LDT is ''not'' saved during a hardware Task Switch, and if you do it the wrong way around, an inopportune interrupt may send your code into oblivion. Alternately, you can disable interrupts while the "foreign" LDT is in place - and hope that you don't get an exception!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/User/User|User]]/LDT.inc</code> ==
<source lang="asm">
;
; Exec/User/LDT.inc
;

; This module creates a User-mode Program's LDT and populates it.
; For demonstration purpose, it deliberately "forgets" to mark some of the
; Segments as "Present", which should trigger the Segment Not Present exception.

Exec.User.LDT:
                ; Allocate an LDT
                MOV             ECX,User.LDT.Size       ; Need this many bytes
                CALL            Exec.Alloc.LDT
                TEST            EAX,EAX                 ; Any left?
                JZ              .Failed                 ; Nope! Finished

                MOV             [%$LDT],EAX             ; Want to keep this!

                ; Allocate Stack0
                MOV             ECX,User.Stack0.Size    ; Get some RAM for Stack0
                CALL            Exec.Alloc.RAM
                TEST            EAX,EAX                 ; Any left?
                JZ              .Failed                 ; Nope!

                MOV             DL,Type.Mem(Stack, DPL0, RW)
                MOV             DH,Gran.Mem(Byte, Small)
                CALL            Exec.Alloc.LDT.Mem      ; Allocate LDT Entry
                CMP             EAX,User.LDT.Stack0
                JNE             .Failed

                ; Allocate Stack3
                MOV             ECX,User.Stack3.Size    ; Get some RAM for Stack3
                CALL            Exec.Alloc.RAM
                TEST            EAX,EAX                 ; Any left?
                JZ              .Failed                 ; Nope!

                MOV             DL,Type.Mem(Stack, DPL3, RW)
                MOV             DH,Gran.Mem(Byte, Small)
                CALL            Exec.Alloc.LDT.Mem      ; Allocate LDT Entry
                CMP             EAX,User.LDT.Stack3
                JNE             .Failed

                ; Allocate Data
                MOV             ECX,User.Data.Size      ; Get some RAM for Data
                CALL            Exec.Alloc.RAM
                TEST            EAX,EAX                 ; Any left?
                JZ              .Failed                 ; Nope!

                MOV             DL,Type.Mem(Data, DPL3, RW); & ~x86.Desc.Type.Present
                MOV             DH,Gran.Mem(Byte, Big)
                CALL            Exec.Alloc.LDT.Mem      ; Allocate LDT Entry
                CMP             EAX,User.LDT.Data
                JNE             .Failed

                ; Allocate Code
                MOV             EAX,User.Base           ; Assembled User code
                MOV             ECX,User.Size
                MOV             DL,Type.Mem(Code, DPL3, NoRW); & ~x86.Desc.Type.Present
                MOV             DH,Gran.Mem(Byte, Def32)
                CALL            Exec.Alloc.LDT.Mem      ; Allocate LDT Entry
                CMP             EAX,User.LDT.Code
                JNE             .Failed

                ; Now enable the LDT
                CALL            Exec.Alloc.LDT.Enable   ; Finished!

                MOV             EAX,[%$LDT]             ; LDT descriptor
                JMP             .End
.Failed:
                XOR             EAX,EAX
.End:
                RET
</source>
````

<a id="w003568"></a>

## W003568 — User:Johnburger/Demo/Exec/User/TSS

Bron-ID: W003568
Namespace: User (2)
Revisie: 21422 | Datum: 2017-08-29T09:19:30Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/User/TSS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FUser%2FTSS&oldid=21422

````text
There's not much to this code, since <code>[[User:Johnburger/Demo/Exec/Alloc/TSS|Demo/Exec/Alloc/TSS]]</code> does most of the hard work for me. After <code>CALL</code>ing it, the rest of the code initialises the other important registers, as well as filling in the User Code's expected parameters, and then activates the TSS.

That's it! The User Code is now ready to run!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/User/User|User]]/TSS.inc</code> ==
<source lang="asm">
;
; Exec/User/TSS.inc
;

; This module creates a User-mode Program's TSS and populates it.

Exec.User.TSS:
; This function creates a TSS for the LDT, and populates it ready for a User Task
; Input:  AX  = LDT
; Output: EAX = Non-zero on success

                CALL            Exec.Alloc.TSS
                TEST            EAX,EAX
                JZ              .End

; Initialise fields in new TSS for User Task
                MOV             CL,Window.Width
                MOV             CH,Window.Height

                MOV             EDX,[%$Pos]

                MOV     WORD    [ES:x86.TSS.CS],  Selector(User.LDT.Code,   LDT, RPL3)
                MOV    DWORD    [ES:x86.TSS.EIP], User.Entry
                MOV     WORD    [ES:x86.TSS.SS0], Selector(User.LDT.Stack0, LDT, RPL0)
;               MOV    DWORD    [ES:x86.TSS.ESP0],0
                MOV     WORD    [ES:x86.TSS.SS],  Selector(User.LDT.Stack3, LDT, RPL3)
;               MOV    DWORD    [ES:x86.TSS.ESP], 0
                MOV     WORD    [ES:x86.TSS.ES],  Selector(GDT.VGA,         GDT, RPL3)
                MOV     WORD    [ES:x86.TSS.DS],  Selector(User.LDT.Data,   LDT, RPL3)
                MOV    DWORD    [ES:x86.TSS.EBX], VGA.Cols * 2
                MOV             [ES:x86.TSS.ECX], ECX
                MOV             [ES:x86.TSS.EDX], EDX

                MOV             AH,0            ; Not a System TSS
                CALL            Exec.Alloc.TSS.Enable
.End:
                RET
</source>
````

<a id="w003569"></a>

## W003569 — User:Johnburger/Demo/Pad/Sizes

Bron-ID: W003569
Namespace: User (2)
Revisie: 21366 | Datum: 2017-08-28T21:14:26Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/Sizes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FSizes&oldid=21366

````text
Each of the Segments in their code may need a separate <code>.Base</code>, <code>.Size</code> and <code>.Limit</code> defined for them, so that various fields in various system structures can be filled in. Also, we need to know sizes so that the Loader can work out how much it has to load. This is accumulated inside the <code>Demo.Size</code> variable.

This module simply re-opens each of the Segments defined so far in the system, and defines the necessary symbols. Note that the GDT is handled specially: it will grow through the system's lifetime, so we need to know its current <code>.Limit</code>, but not get confused with its final full <code>.Size</code>.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/Sizes.inc</code> ==
<source lang="asm">
;
; Pad/Sizes.inc
;

; Calculate final Real Segment size
                SEGMENT         Real ALIGN=16

Real.Base       EQU             BIOS.Entry

                ALIGN           16

Real.Size       EQU             $ - $$

%assign         Demo.Size       Demo.Size + Real.Size
;-------------------------------------------------------------------------------
; Calculate final IDT Segment size
                SEGMENT         IDT  ALIGN=16

IDT.Start       EQU             Code.Base - IDT.Size

IDT.Limit       EQU             $ - $$ - 1

                ALIGN           16, DB 0

IDT.Size        EQU             $ - $$

%assign         Demo.Size       Demo.Size + IDT.Size
;-------------------------------------------------------------------------------
; Calculate final Ints Segment size
                SEGMENT         Ints  ALIGN=16

Ints.Base       EQU             IDT.Start + IDT.Size ; Equivalent to Code.Base

Ints.Limit      EQU             $ - $$ - 1

                ALIGN           16

Ints.Size       EQU             $ - $$

%assign         Demo.Size       Demo.Size + Ints.Size
;-------------------------------------------------------------------------------
; Calculate final User.LDT Segment size
                SEGMENT         User.LDT  ALIGN=16  NOBITS

User.LDT.Limit  EQU             $ - $$ - 1

                ALIGNB          16

User.LDT.Size   EQU             $ - $$
;-------------------------------------------------------------------------------
; Calculate final User.Data Segment size
                SEGMENT         User.Data  ALIGN=16  NOBITS

User.Data.Limit EQU             $ - $$ - 1

                ALIGNB          16

User.Data.Size  EQU             $ - $$
;-------------------------------------------------------------------------------
; Calculate final User.Code Segment size
                SEGMENT         User  ALIGN=16

User.Base       EQU             Ints.Base + Ints.Size

User.Limit      EQU             $ - $$ - 1

                ALIGN           16

User.Size       EQU             $ - $$

%assign         Demo.Size       Demo.Size + User.Size
;-------------------------------------------------------------------------------
; Calculate final Exec Segment size
                SEGMENT         Exec  ALIGN=16

Exec.Base       EQU             User.Base + User.Size

Exec.Limit      EQU             $ - $$ - 1

                ALIGN           16

Exec.Size       EQU             $ - $$

%assign         Demo.Size       Demo.Size + Exec.Size
;-------------------------------------------------------------------------------
; Calculate final Exec.LDT Segment size
                SEGMENT         Exec.LDT  ALIGN=16

Exec.LDT.Base   EQU             Exec.Base + Exec.Size

Exec.LDT.Limit  EQU             $ - $$ - 1

                ALIGN           16, DB 0

Exec.LDT.Size   EQU             $ - $$

%assign         Demo.Size       Demo.Size + Exec.LDT.Size
;-------------------------------------------------------------------------------
; Calculate final GDT Segment size
                SEGMENT         GDT  ALIGN=16

GDT.Base        EQU             Exec.LDT.Base + Exec.LDT.Size

GDT.Limit       EQU             $ - $$ - 1

; Note that GDT grows from here - no alignment necessary

%assign         Demo.Size       Demo.Size + GDT.Limit + 1
</source>
````

<a id="w003570"></a>

## W003570 — User:Johnburger/Demo/Pad/HardDisk

Bron-ID: W003570
Namespace: User (2)
Revisie: 21367 | Datum: 2017-08-28T21:14:37Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/HardDisk
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FHardDisk&oldid=21367

````text
A HardDisk image doesn't care how little or big it is, as long as it is an even multiple of Sector size. To that end the following has some round-up calculations, and then emits precisely the correct number of <code>00h</code> bytes to pad it to the next size up.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/HardDisk.inc</code> ==
<source lang="asm">
;
; Pad/HardDisk.inc
;

; This file performs the necessary padding to generate a HardDisk image file.
; All of the hard work has been done: all we need to do is generate sufficient
; extra bytes to round the resultant output file to the next Sector Size.

                SEGMENT         HardDisk

; Take the current Size for the program, and round it UP to the next Sector.Size
HardDisk.Last   EQU             (Demo.Size + BIOS.Disk.Sector.Size - 1) & ~(BIOS.Disk.Sector.Size - 1)
HardDisk.Padding EQU            HardDisk.Last - Demo.Size

TIMES           HardDisk.Padding DB 00h

HardDisk.Size   EQU             $ - $$

%assign         Demo.Size       Demo.Size + HardDisk.Size
</source>
````

<a id="w003571"></a>

## W003571 — User:Johnburger/Demo/Pad/Floppy

Bron-ID: W003571
Namespace: User (2)
Revisie: 21368 | Datum: 2017-08-28T21:14:48Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/Floppy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FFloppy&oldid=21368

````text
Padding for a Floppy is even easier than for a [[User:Johnburger/Demo/Pad/HardDisk|HardDisk]] - all we need to know is the desired final size.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/Floppy.inc</code> ==
<source lang="asm">
;
; Pad/Floppy.inc
;

; This file performs the necessary padding to generate a .flp or .img file.
; All of the hard work has been done: all we need to do is generate
; sufficient extra bytes to make the resultant output file large enough to
; satisfy programs like earlier versions VMware.
; These programs use the file size to determine the size of the simulated floppy,
; and from there the geometry to use (number of sectors per track, number of
; tracks per side, etc.)

                SEGMENT         Floppy

;Floppy.Format   EQU             720
;Floppy.Format   EQU             1200
Floppy.Format   EQU             1440
;Floppy.Format   EQU             2880

Floppy.Padding  EQU             Floppy.Format*1024 - Demo.Size ; From kiB to bytes

TIMES           Floppy.Padding  DB 00h

Floppy.Size     EQU             $ - $$

%assign         Demo.Size       Demo.Size + Floppy.Size
</source>
````

<a id="w003572"></a>

## W003572 — User talk:Johnburger/Demo/Overview

Bron-ID: W003572
Namespace: User talk (3)
Revisie: 21151 | Datum: 2017-08-28T20:20:32Z
Bron: https://wiki.osdev.org/User_talk:Johnburger/Demo/Overview
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AJohnburger%2FDemo%2FOverview&oldid=21151

````text
: [[User:Johnburger|John Burger]] 03:02, 14 April 2014 (CDT) Let me say this up front: yes, I know this "Overview" page is big - one of the biggest on the Wiki, in fact. But what I wanted ''most'' for this page was for it to be a one-stop reference for any of the subjects that this Demonstrator uses - and that's represented by the Table of Contents. The small description for each subtitle is a lead-in to the Main Article for that subject, which is where the source code is.
: [[User:Johnburger|John Burger]] 03:02, 14 April 2014 (CDT) I am willing to re-lay out the whole thing, just as soon as someone comes up with a suggestion that I'm happy with. Sorry, but I'm not happy with the idea of extracting each major subject into its own page, to reduce the size of this one: that will defeat the purpose of the Table of Contents.
:: [[User:Johnburger|John Burger]] 03:02, 14 April 2014 (CDT) One solution I might entertain would involve this page having nothing but the hierarchy of headings, and then some kind of #REDIRECT for each of the resultant #sections - but I don't think #REDIRECT works like that.
````

<a id="w003573"></a>

## W003573 — User:Johnburger/Demo/Exec/Alloc/Alloc

Bron-ID: W003573
Namespace: User (2)
Revisie: 21369 | Datum: 2017-08-28T21:15:00Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/Alloc
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FAlloc&oldid=21369

````text
You can Alloc RAM, but without a Descriptor, you've got nothing!

You can modify existing DT entries in the IDT, GDT or LDT.

You can Alloc a new DT entry in either the LDT or GDT.

Or you can Alloc a new TSS.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Overview#Allocator_Functions|Alloc]]/Alloc.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/Alloc.inc
;

; This module handles allocating various system resources in separate helper
; functions.
;
; One sub-module is called DT.inc - it handles allocating in both GDTs and LDTs,
; but requires some setup first: this is performed in the appropriate sub-module.
; As such, it's only to be used internally: alas, assembler doesn't have a
; "private:" designator!

;-------------------------------------------------------------------------------
%include        "Exec/Alloc/RAM.inc"    ; RAM-related functions
;-------------------------------------------------------------------------------
%include        "Exec/Alloc/DT.inc"     ; Descriptor Table helper functions
;-------------------------------------------------------------------------------
%include        "Exec/Alloc/IDT.inc"    ; IDT-related functions
;-------------------------------------------------------------------------------
%include        "Exec/Alloc/GDT.inc"    ; GDT-related functions
;-------------------------------------------------------------------------------
%include        "Exec/Alloc/LDT.inc"    ; LDT-related functions
;-------------------------------------------------------------------------------
%include        "Exec/Alloc/TSS.inc"    ; TSS-related functions
</source>
````

<a id="w003574"></a>

## W003574 — User:Johnburger/Demo/Exec/Alloc/RAM

Bron-ID: W003574
Namespace: User (2)
Revisie: 21433 | Datum: 2017-08-29T09:27:42Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/RAM
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FRAM&oldid=21433

````text
This module contains two functions:

=== <code>Exec.Alloc.RAM</code> ===
This function allocates the passed-in number of bytes, and returns its linear address. Note that this RAM is not yet accessible: you need a Descriptor for that! But at least you now have the Base and Size parameters to create one.

Please note that this is a na&iuml;ve implementation: it merely adds the requested size to the current top of RAM, and returns the previous value. It is intelligent enough to jump the 640 kiB to 1 MiB device/ROM/BIOS hole, but it doesn't even know where the ultimate top of RAM is! That has been left as an exercise for the reader...

=== <code>Exec.Alloc.RAM.Zero</code> ===
This does what it says: zeroes the passed-in number of bytes starting from the passed-in pointer.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Alloc/Alloc|Alloc]]/RAM.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/RAM.inc
;

; This module provides RAM-related functions, such as Allocating and Zeroing
; blocks of RAM. It could use the RAM Map discovered from the BIOS during Boot,
; but that's been left as an exercise for the reader!
;
; Instead, the allocator merely keeps track of the last allocated address as a
; "high water mark", returning zero if that mark becomes greater than what's
; available. Of course, this naïve implementation merely keeps giving more and
; more - with enough intelligence to skip from RAMTop to the 1 MiB mark.

;...............................................................................
; This function allocates a block of RAM from the available pool.
; Note that it doesn't allocate a Descriptor for it, since it doesn't know
; what type it'll be, nor which Table to put it in.
;
; Also note that this implementation is NOT thread-safe. It assumes that only
; one Task will ever be accessing it at the same time. To work around that,
; either some kind of lock will be required, or atomic operations such as used
; in the 80486 (such as XADD) should be used.
;
; Input:  DS  = Data
;         ECX = Size requested
; Output: EAX = Address of allocated RAM, or zero if insufficient.
;         ECX preserved
Exec.Alloc.RAM:
                MOV             EAX,[Data.RAM]    ; Get current high-water mark
.Retry:
                LEA             EDX,[EAX+ECX+0Fh] ; Get Top, rounded to paragraph
                AND             EDX,~0Fh          ; Well, it is now!
                CMP             EDX,[Data.RAMTop] ; Too far?
                JBE             .Save             ; Not yet...

                MOV             EAX,0010_0000h    ; Skip to 1 MiB
                MOV    DWORD    [Data.RAMTop],-1  ; And pretend the rest is ours!
                JMP             .Retry            ; And try again
.Save:
                MOV             [Data.RAM],EDX    ; Save new high-water mark
.End:
                RET

;...............................................................................
; This function zeroes out the block of passed-in RAM.
; Input:  ES  = Segment to zero
;         ECX = Size to zero
;         EDI = Where to zero
; Output: ES and ECX preserved.
;         EAX and EDI? Not so much!
Exec.Alloc.RAM.Zero:
                PUSH            ECX             ; As promised...

                CLD                             ; Work forwards

                XOR             EAX,EAX         ; Store zeroes
                SHR             ECX,2           ; Do DWORDs initially
                REP             STOSD           ; Zero out whole DWORDs at a time

                MOV             ECX,[ESP]       ; Recover saved value
                AND             ECX,03h         ; Mask off DWORDs already done
                REP             STOSB           ; Make up any last bytes

                POP             ECX
                RET
</source>
````

<a id="w003575"></a>

## W003575 — User:Johnburger/Demo/Exec/Alloc/DT

Bron-ID: W003575
Namespace: User (2)
Revisie: 21432 | Datum: 2017-08-29T09:27:32Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/DT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FDT&oldid=21432

````text
This module has three functions:

=== <code>Exec.Alloc.DT</code> ===
This function allocates a new Descriptor in either the GDT or LDT. It uses the same algorithm in both cases, since they've been set up with similar Allocator Data in their first entry (which is not used in the GDT, but is a small waste for an LDT. Ah well!)

=== <code>Exec.Alloc.DT.Sys</code> ===
This function modifies an existing DT entry to become a <code>Desc.Sys</code>.
'''''NOTE THAT ANY SEGMENT REGISTER CONTAINING THIS DESCRIPTOR WILL CAUSE A FAULT!'''''
Although a Fault is only generated when a Descriptor is loaded, that is PRECISELY what happens during the return from an interrupt!

In other words, it's NOT safe to keep it in a Segment Register, even if "only for a little while..."

=== <code>Exec.Alloc.DT.Mem</code> ===
This function modifies an existing DT entry to become a <code>Desc.Mem</code>. Any Segment register containing the old value needs to be refreshed. Note that an interrupt WILL re-load a segment register: you cannot safely assume that a "stale" Segment register still uses the old data!

You'll note that this code handles the conversion of the passed-in size into a Limit - which may be more than just subtracting one from it. It also converts it for [[Expand&nbsp;Down]] Segments, and Granularity if the Size is too large.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Alloc/Alloc|Alloc]]/DT.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/DT.inc
;

; This module provides generic functions for any of the Descriptor Tables:
; GDT, LDT or IDT. Only the first two have the capacity for new Descriptors
; to be added though.

;-------------------------------------------------------------------------------
; This function allocates a new Descriptor in either the GDT or LDT. It uses the
; same algorithm in both cases, since they've been set up with similar Allocator
; Data in their first entry (which is not used in the GDT, but is a small waste
; for an LDT. Ah well!)
;
; The algorithm is simple:
; 1) If the .Free entry is non-zero, that will be the returned Descriptor.
;    First though, load the .Free's '.Free' entry, since that is the next
;    available entry (could be zero), and store that in .Free instead.
; 2) If the .Free entry is zero, then check .Limit. If that is -1, the table is
;    FULL! Return a zero Descriptor.
; 3) Otherwise, add Descriptor size to .Limit and return the old value - don't
;    forget to add 1 though, since it was a Limit.
;
; Input:  ES  = Descriptor Table to allocate from
; Output: EAX = Descriptor allocated, or zero if Table is full
;         EDI modified

Exec.Alloc.DT:
                MOVZX           EAX,WORD [ES:DT.Alloc.Free] ; Get potential Free one
                TEST            EAX,EAX                 ; Any?
                JZ              .Grow                   ; No, so grow DT

                MOVZX           EDI,WORD [ES:EAX + DT.Alloc.Free] ; Get Free's next
                MOV             [ES:DT.Alloc.Free],EDI  ; Store as new Free
                JMP             .End                    ; Done!
.Grow:
                MOV             AX,[ES:DT.Alloc.Limit]  ; Get current Limit
                INC             AX                      ; Add one
                JZ              .End                    ; Uh oh! Full!

                ADD     WORD    [ES:DT.Alloc.Limit],x86.Desc_size ; Increase Limit
.End:
                RET

;-------------------------------------------------------------------------------
; This function modifies an existing DT entry to become a Desc.Sys.
; NOTE THAT ANY SEGMENT REGISTER CONTAINING THIS DESCRIPTOR WILL CAUSE A FAULT!
; Although a Fault is only generated when a Descriptor is loaded, that is
; PRECISELY what happens during the return from an interrupt!
; In other words, it's NOT safe to keep it, even if "only for a little while..."
; Input:  ES:EBX = Descriptor Table entry to modify
;         CX:EAX = Selector : Offset of System entry
;         DL     = Descriptor Type
; Output: EAX modified

Exec.Alloc.DT.Sys:
                MOV             [ES:EBX + x86.Desc.Sys.OffsetLo],AX
                SHR             EAX,16
                MOV             [ES:EBX + x86.Desc.Sys.Selector],CX
                MOV             [ES:EBX + x86.Desc.Sys.OffsetHi],AX
                MOV             [ES:EBX + x86.Desc.Sys.Type],DL

                RET

;-------------------------------------------------------------------------------
; This function modifies an existing DT entry to become a Desc.Mem.
; Any Segment register containing the old value needs to be refreshed.
; Note that an interrupt WILL re-load a segment register: you cannot safely
; assume that a "stale" Segment register still uses the old data!
; Input:  ES:EBX = Descriptor Table entry to modify
;         EAX    = Base
;         ECX    = Size (Converted to Limit)
;         DL     = Descriptor Type
;         DH     = Descriptor Granularity
; Output: EAX, ECX and DH modified

Exec.Alloc.DT.Mem:
                TEST            DL,x86.Desc.Mem.Type.Code
                JNZ             .Scale          ; Code cannot be Expand-Down
                TEST            DL,x86.Desc.Mem.Type.Expand
                JZ              .Scale          ; Not Expand-Down either
                ADD             EAX,ECX         ; It is! Add Size to Base
                NEG             ECX             ; And negate ECX to exclude size
                TEST            DH,x86.Desc.Mem.Gran.Big ; 32-bit?
                JNZ             .Scale          ; Yes
                AND             ECX,0000_FFFFh  ; No. Bring down to 16-bit range
                SUB             EAX,0001_0000h  ; And alter Base to suit
.Scale:
                DEC             ECX             ; Convert to Limit
                CMP             ECX,000F_FFFFh  ; Will Limit fit?
                JBE             .Set            ; Yep, so do so
                SHR             ECX,12          ; Nope! Accommodate it
                OR              DH,x86.Desc.Mem.Gran.Gran
.Set:
                MOV             [ES:EBX + x86.Desc.Mem.LimitLo],CX
                SHR             ECX,16
                MOV             [ES:EBX + x86.Desc.Mem.BaseLo], AX
                SHR             EAX,16
                MOV             [ES:EBX + x86.Desc.Mem.BaseMid],AL
                OR              DH,CL
                MOV             [ES:EBX + x86.Desc.Mem.Type],   DL
                MOV             [ES:EBX + x86.Desc.Mem.Gran],   DH
                MOV             [ES:EBX + x86.Desc.Mem.BaseHi], AH

                RET
</source>
````

<a id="w003576"></a>

## W003576 — User:Johnburger/Demo/Exec/Alloc/IDT

Bron-ID: W003576
Namespace: User (2)
Revisie: 21431 | Datum: 2017-08-29T09:27:22Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/IDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FIDT&oldid=21431

````text
This module provides a function which modifies an Entry in the Interrupt Descriptor Table.

Note that the IDT should only ever hold System Descriptors - Memory Descriptors in the IDT don't make any sense!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Alloc/Alloc|Alloc]]/IDT.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/IDT.inc
;

; This module provides functions to modify the IDT.

;-------------------------------------------------------------------------------
; This function modifies an IDT entry.
; Input:  EBX    = IDT Descriptor entry
;         CX:EAX = Selector : Offset of handler
;         DL     = Descriptor Type
; Output: ES, EAX, EDI modified

Exec.Alloc.IDT:
                MOV             DI,Selector(Exec.LDT.IDT, LDT, RPL0)
                MOV             ES,DI           ; Point to IDT with ES

                CALL            Exec.Alloc.DT.Sys
                RET
</source>
````

<a id="w003577"></a>

## W003577 — User:Johnburger/Demo/Exec/Alloc/GDT

Bron-ID: W003577
Namespace: User (2)
Revisie: 21429 | Datum: 2017-08-29T09:23:54Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/GDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FGDT&oldid=21429

````text
This module provides two functions to allocate either a new <code>Desc.Sys</code> Entry or a new <code>Desc.Mem</code> Entry to the GDT.

The functions use those found in [[User:Johnburger/Demo/Exec/Alloc/DT|Demo/Exec/Alloc/DT]], which convert the passed-in parameters as necessary to make them fit inside the new Entry.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Alloc/Alloc|Alloc]]/GDT.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/GDT.inc
;

; This module provides functions to allocate entries in the GDT.

;-------------------------------------------------------------------------------
; This function allocates a new Desc.Sys in the GDT
; Input:  CX:EAX = Selector : Offset of handler
;         DL     = Descriptor Type
; Output: EAX    = Allocated Descriptor, or zero on error
;         EAX, EBX, EDI modified

Exec.Alloc.GDT.Sys:
                MOV             DI,GS           ; Get GDT selector
                MOV             ES,DI           ; Into ES for Alloc.DT

                PUSH            EAX             ; Save for later
                CALL            Exec.Alloc.DT   ; Allocate an Entry
                MOV             EBX,EAX         ; This is Descriptor to modify

                TEST            EAX,EAX         ; Any left?
                POP             EAX             ; It's later!
                JZ              .Finish         ; Uh oh!

                CALL            Exec.Alloc.DT.Sys ; Call helper function

                LGDT            [ES:GDT.Alloc.Pseudo] ; Update GDT size
.Finish:
                MOV             EAX,EBX         ; Return value
                RET

;-------------------------------------------------------------------------------
; This function allocates a new Desc.Mem in the GDT
; Input:  EAX    = Base
;         ECX    = Size (Converted to Limit)
;         DL     = Descriptor Type
;         DH     = Descriptor Granularity
; Output: EAX    = Allocated Descriptor, or zero on error
;         ECX, DH, EDI modified

Exec.Alloc.GDT.Mem:
                MOV             DI,GS           ; Get GDT selector
                MOV             ES,DI           ; Into ES for Alloc.DT

                PUSH            EAX             ; Save for later
                CALL            Exec.Alloc.DT   ; Allocate an Entry
                MOV             EBX,EAX         ; This is Descriptor to modify

                TEST            EAX,EAX         ; Any left?
                POP             EAX             ; It's later!
                JZ              .Finish         ; Uh oh!

                CALL            Exec.Alloc.DT.Mem ; Call helper function

                LGDT            [ES:GDT.Alloc.Pseudo] ; Update GDT size
.Finish:
                MOV             EAX,EBX         ; Return value
                RET
</source>
````

<a id="w003578"></a>

## W003578 — User:Johnburger/Demo/Exec/Alloc/LDT

Bron-ID: W003578
Namespace: User (2)
Revisie: 21428 | Datum: 2017-08-29T09:23:40Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/LDT
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FLDT&oldid=21428

````text
This module provides four functions, to allocate LDTs and then to allocate Entries inside them.
__TOC__
Allocating the memory for a new LDT is easy, but the code assumes that the LDT will need some tailoring done to it before it will become a true LDT. For that reason <code>Exec.Alloc.LDT</code> returns a Data Segment, and expects <code>Exec.Alloc.LDT.Enable</code> to be <code>CALL</code>ed when finished so that it can convert it to a true LDT.

The last two functions use those found in [[User:Johnburger/Demo/Exec/Alloc/DT|Demo/Exec/Alloc/DT]], which convert the passed-in parameters as necessary to make them fit inside the new Entry.

=== <code>Exec.Alloc.LDT</code> ===
This function allocates a new Data Segment with the passed-in size, and sets up the first couple of Entries in the nascent LDT for future Allocate purposes.

=== <code>Exec.Alloc.LDT.Enable</code> ===
This function converts the Data Segment into a true LDT.

=== <code>Exec.Alloc.LDT.Sys</code> ===
This Allocates a new System Descriptor in the LDT.

=== <code>Exec.Alloc.LDT.Mem</code> ===
This Allocates a new Memory Descriptor in the LDT.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Alloc/Alloc|Alloc]]/LDT.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/LDT.inc
;

; This module provides functions to create LDTs and allocate entries in them.

;-------------------------------------------------------------------------------
; This function:
; 1) Allocates the requested RAM for an LDT;
; 2) Fills in the first entry with LDT.Alloc;
; 3) Fills in the second entry with an alias for this LDT (useful for debugging)
; 4) Returns a Data Descriptor for the LDT, ready for more customisation.
; To actually turn it into an LDT, call Exec.Alloc.LDT.Enable.
;
; Input:  ECX = Size
; Output: AX  = GDT Descriptor for (future) LDT, or zero on error
;         ES  = GDT Descriptor unless AX is zero
Exec.Alloc.LDT:

%push Alloc.LDT ; Let's not leave these %defines just lying around...
%define         %$Base          EBP - 4
%define         %$Size          EBP - 8

                ENTER           8, 0

                MOV             [%$Size], ECX

                CALL            Exec.Alloc.RAM
                TEST            EAX,EAX          ; Any left?
                JZ              .End             ; Pity that!

                MOV             [%$Base],EAX

                MOV             EBX,EAX          ; Base to allocate
                MOV             DL,Type.Mem(Data, DPL0, RW)
                MOV             DH,Gran.Mem(Byte, Small)
                CALL            Exec.Alloc.GDT.Mem ; Get Descriptor to use
                TEST            EAX,EAX          ; Any left?
                JZ              .End             ; Pity that! **** Memory leak! ****

                MOV             ES,AX            ; Fresh Descriptor!
                MOV             ECX,[%$Size]     ; Bytes to zero
                XOR             EDI,EDI          ; Zero everything
                CALL            Exec.Alloc.RAM.Zero

                ; Set up first entry as an LDT.Alloc
                MOV     WORD    [ES:LDT.Alloc.Limit],x86.Desc_size - 1
                MOV     BYTE    [ES:LDT.Alloc.Type],x86.Desc.Sys.Type.LDT
                MOV             [ES:LDT.Alloc.GDT],ES

                ; Set up second entry as an alias for this very LDT
                MOV             EAX,[%$Base]            ; Pass Base in
                MOV             ECX,[%$Size]            ; Size
                MOV             DL,Type.Mem(Data, DPL0, RW)
                MOV             DH,Gran.Mem(Byte, Small)
                CALL            Exec.Alloc.LDT.Mem      ; Allocate LDT entry

                MOV             AX,ES                   ; Return value
.End:
                LEAVE
%pop
                RET
;-------------------------------------------------------------------------------
; This function converts a Segment previously allocated with Alloc.LDT (above)
; into a true LDT.
;
; Input:  ES = Descriptor to modify
; Output: ES, EBX modified
Exec.Alloc.LDT.Enable:

; Note that I could use GS: overrides to access the GDT, but ES points to the
; soon-to-be LDT - a Bad Thing. This fixes that!
                MOV             EBX, ES         ; Get Descriptor to modify

                MOV             AX,GS           ; Get GDT into ES
                MOV             ES,AX

                MOV     BYTE    [ES:EBX+x86.Desc.Type],Type.Sys(LDT, DPL0, 286)

                RET
;-------------------------------------------------------------------------------
; This function allocates a new Desc.Sys in an LDT
; Input:  ES     = LDT to modify
;         CX:EAX = Selector : Offset of handler
;         DL     = Descriptor Type
; Output: EAX    = Allocated Descriptor, or zero on error
;         EBX, ESI, EDI modified

Exec.Alloc.LDT.Sys:
                PUSH            EAX             ; Save for later
                CALL            Exec.Alloc.DT   ; Allocate an Entry
                MOV             EBX,EAX         ; This is Descriptor to modify

                TEST            EAX,EAX         ; Any left?
                POP             EAX             ; It's later!
                JZ              .Finish         ; Uh oh!

                CALL            Exec.Alloc.DT.Sys ; Call helper function

.Finish:
                MOV             EAX,EBX         ; Return value
                RET

;-------------------------------------------------------------------------------
; This function allocates a new Desc.Mem in an LDT
; Input:  ES     = LDT to modify
;         EAX    = Base
;         ECX    = Size (Converted to Limit)
;         DL     = Descriptor Type
;         DH     = Descriptor Granularity
; Output: EAX    = Allocated Descriptor, or zero on error
;         EBX, ECX, DH, ESI, EDI modified

Exec.Alloc.LDT.Mem:
                PUSH            EAX             ; Save for later
                CALL            Exec.Alloc.DT   ; Allocate an Entry
                MOV             EBX,EAX         ; This is Descriptor to modify

                TEST            EAX,EAX         ; Any left?
                POP             EAX             ; It's later!
                JZ              .Finish         ; Uh oh!

                CALL            Exec.Alloc.DT.Mem ; Call helper function

.Finish:
                MOV             EAX,EBX         ; Return value
                RET
</source>
````

<a id="w003579"></a>

## W003579 — User:Johnburger/Demo/Exec/Alloc/TSS

Bron-ID: W003579
Namespace: User (2)
Revisie: 22424 | Datum: 2018-06-04T01:05:21Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Exec/Alloc/TSS
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FExec%2FAlloc%2FTSS&oldid=22424

````text
This module provides two routines to Allocate a new TSS.

Allocating the memory for a new TSS is easy, but the code assumes that the TSS will need some tailoring done to it before it will become a true TSS. For that reason <code>Exec.Alloc.TSS</code> returns a Data Segment, and expects <code>Exec.Alloc.TSS.Enable</code> to be <code>CALL</code>ed when finished so that it can convert it to a true TSS.

You'll note that the code specifies that it initialises three significant fields: <code>EFlags</code>, <code>IOMap</code> and <code>LDT</code>. The last two are obvious - so why is <code>EFlags</code> important?

The two bits that are initialised are the Always-<code>1</code> bit (which isn't strictly necessary: even if it was 0, it would be set to 1 by the CPU) - '''''and''''' the Interrupt Flag (<code>IF</code>) bit. That means that this Task starts with interrupts enabled. What would happen if it started with them disabled? Since a User Task can't use the <code>CLI</code> or <code>STI</code> instructions, it would effectively mean that the User Task would be running with interrupts permanently disabled: not a good thing for an OS! Try it!
* If the User Code uses <code>User.Halt</code> to wait for its timeslice to end, then the CPU will be HALTed with interrupts disabled - effectively zombied. VMWare detects this and stops the emulation. VirtualBox just locks up.
** To protect against this scenario, the <code>User.Halt</code> system implementation should ensure that interrupts are enabled (<code>STI</code>) before <code>HLT</code> is called. After all, that's what System Code is supposed to do - ensure that User Code can't get the system into strife!
* If the User Code uses <code>User.Yield</code> to end its timeslice prematurely, then the next Task will get its turn - but with no IRQs, neither the Timer nor the Keyboard can interrupt the system. Runaway!
* If the User Code simply does nothing to yield, or just twiddles its thumbs between bouncing updates, then it will be the only task that ever runs - very, very fast!

Another thing to try here is to add <code> | x86.EFlags.TF</code> That will start the Task with [[User:Johnburger/Demo/Overview#Single_Step_Handler|Single Step]] (Trace Flag (<code>TF</code>)) on, and nothing will happen in the task until <code>&lt;Space&gt;</code> is pressed - which will only execute one instruction and then pause again. Pressing <code>&lt;Enter&gt;</code> will remove the Trace Flag and allow the Task to run normally.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Exec/Exec|Exec]]/[[User:Johnburger/Demo/Exec/Alloc/Alloc|Alloc]]/TSS.inc</code> ==
<source lang="asm">
;
; Exec/Alloc/TSS.inc
;

; This module provides functions to allocate and configure Task State Segments.

;-------------------------------------------------------------------------------
; This function allocates sufficient RAM for a TSS, and fills in the most
; important entries: EFlags, IOMap and LDT.
;
; It then returns a Data descriptor, ready for more customisation. To actually
; turn it into a TSS, call Exec.Alloc.TSS.Enable.
;
; Input:  AX  = LDT to use for the TSS
; Output: AX  = GDT Descriptor for (future) TSS, or zero on error
;         ES  = GDT Descriptor unless AX is zero
;         EAX = Memory Base
;         ECX = Memory Size
Exec.Alloc.TSS:

Exec.TSS.EFlags EQU             x86.EFlags.IF | x86.EFlags.1 ; Enable interrupts

%push Alloc.TSS ; Let's not leave these %defines just lying around...
%define         %$LDT           EBP - 4
%define         %$Base          EBP - 8
%define         %$Size          EBP - 12

                ENTER           12, 0

                MOV             [%$LDT],EAX

                MOV             ECX,x86.TSS_size ; Size to allocate
                CALL            Exec.Alloc.RAM
                TEST            EAX,EAX          ; Any left?
                JZ              .End             ; Pity that!

                MOV             [%$Base],EAX
                MOV             [%$Size],ECX

                MOV             DL,Type.Mem(Data, DPL0, RW)
                MOV             DH,Gran.Mem(Byte, Small)
                CALL            Exec.Alloc.GDT.Mem ; Get Descriptor to use
                TEST            EAX,EAX          ; Any left?
                JZ              .End             ; Pity that! **** Memory leak! ****

                MOV             ES,AX            ; Fresh Descriptor!
                MOV             ECX,[%$Size]     ; Number bytes to Zero
                XOR             EDI,EDI          ; Zero everything
                CALL            Exec.Alloc.RAM.Zero

                MOV             AX,[%$LDT]
                MOV             ECX,[%$Size]
                MOV             [ES:x86.TSS.LDT],AX
                MOV             [ES:x86.TSS.IOMap],CX
                MOV    DWORD    [ES:x86.TSS.EFlags],Exec.TSS.EFlags

                MOV             EAX,[%$Base]
.End:
                LEAVE
%pop
                RET

;-------------------------------------------------------------------------------
; This function converts a Segment previously allocated with Alloc.TSS (above)
; into a true TSS. Note that as soon as this happens, the TSS is a candidate for
; switching.
;
; Input:  ES  = Descriptor to modify
;         AH  = System.TSS if TSS is to be a System TSS. Zero otherwise.
; Output: EBX = TSS Descriptor
;         ES modified
Exec.Alloc.TSS.Enable:

; Note that I could use GS: overrides to access the GDT, but ES points to the
; soon-to-be TSS - a Bad Thing. This fixes that!
                MOV             EBX, ES         ; Get Descriptor to modify

                PUSH            GS              ; Get GDT into ES
                POP             ES

                MOV             [ES:EBX+x86.Desc.Type],BYTE Type.Sys(TSS, DPL0, 386)
                OR              [ES:EBX+x86.Desc.Mem.Gran],AH ; OR in System.TSS

                RET
</source>
````

<a id="w003580"></a>

## W003580 — User:Johnburger/Demo/Pad/ISO/RootDir

Bron-ID: W003580
Namespace: User (2)
Revisie: 21370 | Datum: 2017-08-28T21:15:12Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/RootDir
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FRootDir&oldid=21370

````text
Any ISO Directory Sector requires two entries: a pointer to itself, and a pointer to its parent. In the case of the Root Directory, the parent is itself - the name is merely a binary 1 instead of a 0.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/RootDir.inc</code> ==
<source lang="asm">
;
; Pad/ISO/RootDir.inc
;

; This file defines the ISO Root directory. There's not much in it...

ISO.Sector.Root      EQU                ISO.Sectors
%assign              ISO.Sectors        ISO.Sectors + 1

ISO.RootDir:
.Dot            ISO.Dir.Root    ISO.Sector.Root, ISO.Dir.Flag.Dir, 1, 0
.DotDot         ISO.Dir.Root    ISO.Sector.Root, ISO.Dir.Flag.Dir, 1, 1

.Padding        EQU             ISO.Sector.Size - ($ - ISO.RootDir)
                TIMES           .Padding    DB 00h
</source>

(OK, I admit it. I giggle when I pronounce the names of my two directory entry symbols: "dot Dot" and "dot Dot Dot"...)
````

<a id="w003581"></a>

## W003581 — User:Johnburger/Demo/Pad/ISO/ISO

Bron-ID: W003581
Namespace: User (2)
Revisie: 21371 | Datum: 2017-08-28T21:15:28Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/ISO
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FISO&oldid=21371

````text
__TOC__
== Where can this be used? ==
This module can be <code>%include</code>d into any other binary project that has two properties:
# It defines a boot sector in the first 512 bytes:
## The entry is at <code>0000h</code>;
## Location <code>1FEh</code> is <code>55h</code>;
## Location <code>1FFh</code> is <code>AAh</code>.
# It is no more than 32 kiB in size.

== What do I need to do? ==
To successfully assemble, you will need to define the following symbols:
* <code>ISO.Start.Size</code><br>This is the total number of bytes that this module will be appending onto. As noted above, this needs to be less than 32 kiB.
* <code>Version.Program</code><br>A short, one-word name for the program;
* <code>Version.Author</code><br>The author's name - also used as <code>ISO.Primary.Preparer</code>;
* <code>Version.Name</code><br>A longer name for the program;
* <code>Version.Copyright</code><br>A short copyright string;
* <code>Version.String</code><br>A short string representing the Program's version number.

Note that to be truly [[ISO 9660]] compliant, <code>Version.Program</code> should be ALLCAPITALS with no spaces, and the other strings should not contain lower case. Over time these restrictions have been reduced, and just about any characters are acceptable now.

If you want, you can go into [[User:Johnburger/Demo/Pad/ISO/Primary|Demo/Pad/ISO/Primary]] and change their usage there - or perhaps change <code>ISO.Primary.Application</code>.

You would also probably want to modify <code>ISO.Timezone</code> in this file.

=== Example from <code>[[User:Johnburger/Demo/Demo|Demo/Demo]]</code> ===
<source lang="asm">
%define         Version.Program   "Demonstrator"
%define         Version.Author    "John Burger"
%define         Version.Name      Version.Author, "'s 80386 ", Version.Program
%define         Version.Copyright "(c)2014"

%define         Version.Major   1
%define         Version.Minor   0
%define         Version.Build   1000

%defstr         Version.String  %[Version.Major].%[Version.Minor].%[Version.Build]
</source>

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Overview#ISO|ISO]]/ISO.inc</code> ==
<source lang="asm">
;
; Pad/ISO/ISO.inc
;

; This file performs the necessary binary padding to generate a .iso file.
;
; To make an ISO bootable, there needs to be some sectors in defined locations:
; Primary Volume; Boot Volume; Terminator Volume; Boot Catalog; Little-endian
; Path Table; Big-endian Path Table, and finally the Root directory.
; They've been included below in the correct order.
;
; Note that the Boot Catalog can advise the BIOS that it wants to emulate a
; HardDisk, a Floppy, or nothing: quite frankly, this image doesn't care.
;
; Note that according to the original standard, certain strings are VERY
; constrained. These constraints have been relaxed over the years: I'm taking
; advantage. To become more compliant, you will need to restrict your strings.
;
; The biggest limitation to the technique used here is that it will only work if
; the previous image that this module is padding onto is no more than 32 kiB in
; size. This value is defined in ISO.Start.Size

                SEGMENT         ISO

ISO.Timezone    EQU                     40      ; GMT+10:00 (AEST)

%include        "Pad/ISO/Defn.inc"

%assign         ISO.Sectors     16

ISO.Padding     EQU             ISO.Sectors*ISO.Sector.Size - ISO.Start.Size

TIMES           ISO.Padding     DB 00h

%include        "Pad/ISO/Primary.inc"

%include        "Pad/ISO/Boot.inc"

%include        "Pad/ISO/Terminator.inc"

%include        "Pad/ISO/Catalog.inc"

%define         ISO.Endian      Little
%include        "Pad/ISO/PathTable.inc"
%undef          ISO.Endian

%define         ISO.Endian      Big
%include        "Pad/ISO/PathTable.inc"
%undef          ISO.Endian

%include        "Pad/ISO/RootDir.inc"

ISO.Sector.Last      EQU        ISO.Sectors

ISO.Size        EQU             $ - $$
</source>
````

<a id="w003582"></a>

## W003582 — User:Johnburger/Demo/Pad/ISO/Defn

Bron-ID: W003582
Namespace: User (2)
Revisie: 21372 | Datum: 2017-08-28T21:15:40Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/Defn
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FDefn&oldid=21372

````text
This file provides the definitions for the rest of the ISO subsystem. I have tried to make it so that nothing in here needs to be modified for ISO to be used in other projects, and especially for it not to use definitions from the rest of the Demonstrator.

It does however provide quite a number of <code>%define</code>s and <code>%macro</code>s of its own - all with the <code>ISO.</code> prefix to attempt to avoid polluting your namespace!

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/Defn.inc</code> ==
<source lang="asm">
;
; Pad/ISO/Defn.inc
;

; This file holds the definitions for the ISO standard. The biggest oddity is
; their way of catering for both little-endian and big-endian systems out there:
; they avoid it altogether and use a both-endian double-sized structure!

; The following have been defined by the ISO standard
ISO.Sector.Size EQU             2048

ISO.Boot.Media.NoEmulation EQU  0
ISO.Boot.Media.Floppy.1200 EQU  1
ISO.Boot.Media.Floppy.1440 EQU  2
ISO.Boot.Media.Floppy.2880 EQU  3
ISO.Boot.Media.HardDisk    EQU  4

ISO.Boot.Platform.x86      EQU  0
ISO.Boot.Platform.PowerPC  EQU  1
ISO.Boot.Platform.Mac      EQU  2

; For some fields, I need a date and time. Why not now?
%defstr         ISO.Now.Date    __DATE_NUM__    ; YYYYMMDD
%defstr         ISO.Now.Time    __TIME_NUM__    ; HHMMSS

; This extracts sub-parts of the above numbers as integers
%define         ISO.Extract(Number, Mod, Div) ((Number % Mod) / Div)

; The ISO standard caters for both Big- and Little-endian numbers. Here are
; some macros for when it matters which will be used
%define         ISO.Endian.Little.Word(Num)  (Num)
%define         ISO.Endian.Big.Word(Num)     ((Num >> 8) | (Num << 8) & 0FFFFh)

%define         ISO.Endian.Little.DWord(Num) (Num)
%define         ISO.Endian.Big.DWord(Num)    ( ((Num >> 24) &  0000_00FFh) | \
                                               ((Num >>  8) &  0000_FF00h) | \
                                               ((Num <<  8) &  00FF_0000h) | \
                                               ((Num << 24) & 0FF00_0000h)   \
                                             )

; And here are the macros to represent the Both-endian numbers
%macro          ISO.Endian.Word 1
                DW              ISO.Endian.Little.Word(%1)
                DW              ISO.Endian.Big.Word(%1)
%endmacro

%macro          ISO.Endian.DWord 1
                DD              ISO.Endian.Little.DWord(%1)
                DD              ISO.Endian.Big.DWord(%1)
%endmacro

; NASM's __DATE_NUM__ and __TIME_NUM__ macros are already in the correct format!
%macro          ISO.DateTime    0
%00.Date        DB              ISO.Now.Date
%00.Time        DB              ISO.Now.Time
%00.Hundredths  DB              "00"
%00.Timezone    DB              ISO.Timezone
%if ($-%00.Date) != 17
%error "DateTime macro results incorrect"
%endif
%endmacro

; Every Volume sector in the Volume Set needs the same Header to start
%define         ISO.Volume.Header.ID.String "CD001"

ISO.Volume.Header.Type.Boot         EQU     0
ISO.Volume.Header.Type.Primary      EQU     1
ISO.Volume.Header.Type.Supplemetary EQU     2
ISO.Volume.Header.Type.Partition    EQU     3
ISO.Volume.Header.Type.Terminator   EQU   255

                STRUC           ISO.Volume.Header
.Type           RESB            1               ; ISO.Volume.Header.Type.*
.ID             RESB            5               ; ISO.Volume.Header.ID.String
.Version        RESB            1               ; Usually 1
                ENDSTRUC

%macro          ISO.Volume.Header 2
%00             ISTRUC        ISO.Volume.Header
AT ISO.Volume.Header.Type,    DB                %1
AT ISO.Volume.Header.ID,      DB                ISO.Volume.Header.ID.String
AT ISO.Volume.Header.Version, DB                %2
                IEND
%endmacro

; This is the format of an ISO Directory Entry. At least, it nearly is -
; .Identifier is the name, and the whole Entry's size needs to be even
                STRUC           ISO.Dir
.NumBytes       RESB            1
.NumExtSectors  RESB            1
.Sector         RESD            2       ; Both-endian
.Size           RESD            2       ; Both-endian
.Years          RESB            1
.Month          RESB            1
.Day            RESB            1
.Hour           RESB            1
.Min            RESB            1
.Sec            RESB            1
.Timezone       RESB            1
.Flags          RESB            1
.Interleave.Size RESB           1
.Interleave.Gap RESB            1
.Volume.SeqNum  RESW            2       ; Both-endian
.Length         RESB            1
.Identifier     RESB            1
                ENDSTRUC

; These are the most common Entry flags
ISO.Dir.Flag.Normal EQU         0000_0000b
ISO.Dir.Flag.Hidden EQU         0000_0001b
ISO.Dir.Flag.File   EQU         0000_0000b
ISO.Dir.Flag.Dir    EQU         0000_0010b

%macro          ISO.Dir.Root    4
%00             ISTRUC    ISO.Dir
AT ISO.Dir.NumBytes,      DB    ISO.Dir_size
AT ISO.Dir.NumExtSectors, DB    0
AT ISO.Dir.Sector,        ISO.Endian.DWord %1
AT ISO.Dir.Size,          ISO.Endian.DWord ISO.Sector.Size
AT ISO.Dir.Years,         DB    ISO.Extract(__DATE_NUM__,100000000,10000) - 1900
AT ISO.Dir.Month,         DB    ISO.Extract(__DATE_NUM__,10000,100)
AT ISO.Dir.Day,           DB    ISO.Extract(__DATE_NUM__,100,1)
AT ISO.Dir.Hour,          DB    ISO.Extract(__TIME_NUM__,1000000,10000)
AT ISO.Dir.Min,           DB    ISO.Extract(__TIME_NUM__,10000,100)
AT ISO.Dir.Sec,           DB    ISO.Extract(__TIME_NUM__,100,1)
AT ISO.Dir.Timezone,      DB    ISO.Timezone
AT ISO.Dir.Flags,         DB    %2
AT ISO.Dir.Interleave.Size, DB  0
AT ISO.Dir.Interleave.Gap,  DB  0
AT ISO.Dir.Volume.SeqNum, ISO.Endian.Word 1
AT ISO.Dir.Length,        DB    %3
AT ISO.Dir.Identifier,    DB    %4
                IEND
%endmacro

; The Boot Catalog has the only Checksum in the formal ISO specification.
; Here are the macro definitions to get NASM to calculate it for us!
ISO.Validation.Header    EQU    01h
ISO.Validation.Signature EQU    0AA55h

%macro          ISO.Checksum    2
%assign         Checksum        (%2<<8) + ISO.Validation.Header + ISO.Validation.Signature
%strlen         ID.Length       %1
%assign         ID.Pos          1
%rep ID.Length
%substr         ID.High         %1 ID.Pos+1
%substr         ID.Low          %1 ID.Pos+0
%assign         Checksum        Checksum + ((ID.High << 8) | ID.Low)
%assign         ID.Pos          ID.Pos+2
%endrep
%00             DW              (0-Checksum) & 0FFFFh
%endmacro

%macro          ISO.Validation  2
.Header         DB              ISO.Validation.Header
.Platform       DB              %2
.Reserved       DW              0000h
.ID             DB              %1
                TIMES           ((1Bh-04h+1) - ($-.ID)) DB 00h
.Checksum       ISO.Checksum    %1, %2
.Signature      DW              ISO.Validation.Signature
%endmacro
</source>
````

<a id="w003583"></a>

## W003583 — User:Johnburger/Demo/Pad/ISO/Primary

Bron-ID: W003583
Namespace: User (2)
Revisie: 21373 | Datum: 2017-08-28T21:15:52Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/Primary
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FPrimary&oldid=21373

````text
You could have a lot of fun with the huge number of strings in this Sector. To be truly [[ISO 9660]] compliant, none of them should contain lowercase letters, and a couple (<code>ISO.Primary.Volume</code> and <code>ISO.Primary.VolumeSet</code>) should only hold uppercase letters and/or numbers - no punctuation or spaces!

Over time these restrictions have been relaxed, and many systems that understand ISOs will happily work with any ASCII characters.

Note that some fields don't define what you think: they're actually the ISO names of files in the Root Directory that are supposed to contain the indicated information - <code>ISO.Primary.Copyright</code>, <code>ISO.Primary.Abstract</code> and <code>ISO.Primary.Biblio</code>.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/Primary.inc</code> ==
<source lang="asm">
;
; Pad/ISO/Primary.inc
;

; This file defines the Primary Volume Sector for the ISO image.

%define         Version.System          Version.Program, " ", Version.Copyright, " v.", Version.String

%define         ISO.Primary.System      Version.System
%define         ISO.Primary.Volume      Version.Name
%define         ISO.Primary.VolumeSet   ""
%define         ISO.Primary.Publisher   Version.Author
%define         ISO.Primary.Preparer    Version.Author
%define         ISO.Primary.Application "NASM (http://www.nasm.us/) ", \
                                        __NASM_VER__, " ", __DATE__, " ", __TIME__
%define         ISO.Primary.Copyright   ""      ; Name of file in Root
%define         ISO.Primary.Abstract    ""      ; Name of file in Root
%define         ISO.Primary.Biblio      ""      ; Name of file in Root

ISO.Sector.Primary    EQU               ISO.Sectors
%assign               ISO.Sectors       ISO.Sectors + 1

ISO.Volume.Primary:
.Header         ISO.Volume.Header       ISO.Volume.Header.Type.Primary, 1
                DB              00h

.SystemID       DB              ISO.Primary.System ; (Uppercase and punctuation)
                TIMES           (32-($-.SystemID)) DB ' '

.VolumeID       DB              ISO.Primary.Volume
                TIMES           (32-($-.VolumeID)) DB ' ' ; (Uppercase only)

                TIMES           8 DB 00h

.TotalSectors   ISO.Endian.DWord ISO.Sector.Last

                TIMES            32 DB 00h

.VolumeSetSize  ISO.Endian.Word  1              ; Number of Volumes in this Set
.VolumeSeqNum   ISO.Endian.Word  1              ; Which number Volume this is
.LogicalSize    ISO.Endian.Word  ISO.Sector.Size

.PathTableSize.Little DD         ISO.Endian.Little.DWord(ISO.PathTable.Little.Size)
.PathTableSize.Big    DD         ISO.Endian.Big.DWord(ISO.PathTable.Big.Size)
.PathTable.1.Little   DD         ISO.Endian.Little.DWord(ISO.Sector.PathTable.Little)
.PathTable.2.Little   DD         ISO.Endian.Little.DWord(0) ; Optional, so blank
.PathTable.1.Big      DD         ISO.Endian.Big.DWord(ISO.Sector.PathTable.Big)
.PathTable.2.Big      DD         ISO.Endian.Big.DWord(0)    ; Optional, so blank

.RootDirectory  ISO.Dir.Root     ISO.Sector.Root, ISO.Dir.Flag.Dir, 1, 0

.VolumeSetID    DB               ISO.Primary.VolumeSet ; (Uppercase only)
                TIMES            (128-($-.VolumeSetID)) DB ' '

.PublisherID    DB               ISO.Primary.Publisher ; (Uppercase and punctuation)
                TIMES            (128-($-.PublisherID)) DB ' '

.PreparerID     DB               ISO.Primary.Preparer ; (Uppercase and punctuation)
                TIMES            (128-($-.PreparerID)) DB ' '

.ApplicationID  DB               ISO.Primary.Application ; (Uppercase and punctuation)
                TIMES            (128-($-.ApplicationID)) DB ' '

.CopyrightFile  DB               ISO.Primary.Copyright
                TIMES            (37-($-.CopyrightFile)) DB ' '

.AbstractFile   DB               ISO.Primary.Abstract
                TIMES            (37-($-.AbstractFile)) DB ' '

.BiblioFile     DB               ISO.Primary.Biblio
                TIMES            (37-($-.BiblioFile)) DB ' '

.CreateDate     ISO.DateTime
.ModifyDate     ISO.DateTime
.ExpiryDate     DB               "9999123123595999",-48 ; Basically never...
.EffectiveDate  ISO.DateTime

.StructureVer   DB               01h
.Reserved       DB               00h

.PrivateUse     TIMES            512 DB 00h

.Padding        EQU             ISO.Sector.Size - ($ - ISO.Volume.Primary)
                TIMES           .Padding    DB 00h
</source>
````

<a id="w003584"></a>

## W003584 — User:Johnburger/Demo/Pad/ISO/Boot

Bron-ID: W003584
Namespace: User (2)
Revisie: 21374 | Datum: 2017-08-28T21:16:03Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/Boot
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FBoot&oldid=21374

````text
There's not much in this sector: it merely marks the CD as adhering to the [[El-Torito]] standard.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/Boot.inc</code> ==
<source lang="asm">
;
; Pad/ISO/Boot.inc
;

; This file defines the El Torito Boot Record Volume Sector for the ISO image

ISO.Sector.Boot       EQU               ISO.Sectors
%assign               ISO.Sectors       ISO.Sectors + 1

ISO.Volume.Boot:
.Header         ISO.Volume.Header       ISO.Volume.Header.Type.Boot, 1

.SystemID       DB              "EL TORITO SPECIFICATION"
                TIMES           ((26h-07h+1) - ($-.SystemID)) DB 00h

                TIMES           (46h-27h+1) DB 00h

.Catalog        DD              ISO.Sector.Catalog

.Padding        EQU             ISO.Sector.Size - ($ - ISO.Volume.Boot)
                TIMES           .Padding    DB 00h
</source>
````

<a id="w003585"></a>

## W003585 — User:Johnburger/Demo/Pad/ISO/Terminator

Bron-ID: W003585
Namespace: User (2)
Revisie: 21375 | Datum: 2017-08-28T21:16:16Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/Terminator
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FTerminator&oldid=21375

````text
This is the simplest ISO Sector defined: it marks that there are no more Volume Sets to look for.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/Terminator.inc</code> ==
<source lang="asm">
;
; Pad/ISO/Terminator.inc
;

; This file defines the Set Terminator Volume Sector for the ISO image

ISO.Sector.Terminator EQU               ISO.Sectors
%assign               ISO.Sectors       ISO.Sectors + 1

ISO.Volume.Terminator:
.Header         ISO.Volume.Header       ISO.Volume.Header.Type.Terminator, 1

.Padding        EQU             ISO.Sector.Size - ($ - ISO.Volume.Terminator)
                TIMES           .Padding    DB 00h
</source>
````

<a id="w003586"></a>

## W003586 — User:Johnburger/Demo/Pad/ISO/PathTable

Bron-ID: W003586
Namespace: User (2)
Revisie: 21376 | Datum: 2017-08-28T21:16:27Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/PathTable
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FPathTable&oldid=21376

````text
The PathTable Sector isn't complicated when there are no files on the CD!

Don't forget that this file will be <code>%include</code>ed twice: once with <code>ISO.Endian</code> <code>%define</code>d as <code>Little</code>, and then again as <code>Big</code>.

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/PathTable.inc</code> ==
<source lang="asm">
;
; Pad/ISO/PathTable.inc
;

; This file defines the ISO Path Table. There are two similar flavours: Big and
; Little Endian. To that end, this file expects the ISO.Endian macro to be set
; to either Little or Big before being %included.

ISO.Sector.PathTable.%[ISO.Endian] EQU  ISO.Sectors
%assign              ISO.Sectors        ISO.Sectors + 1

ISO.PathTable.%[ISO.Endian]:
.Length         DB              1       ; Number of Sectors in Path Table
.NumExtSectors  DB              0

.Sector         DD              ISO.Endian.%[ISO.Endian].DWord(ISO.Sector.Root)
.Parent         DW              ISO.Endian.%[ISO.Endian].Word(1)

.Name           DB              0
.Pad            DB              0

.Size           EQU             $ - ISO.PathTable.%[ISO.Endian]

.Padding        EQU             ISO.Sector.Size - .Size
                TIMES           .Padding    DB 00h
</source>
````

<a id="w003587"></a>

## W003587 — User:Johnburger/Demo/Pad/ISO/Catalog

Bron-ID: W003587
Namespace: User (2)
Revisie: 21377 | Datum: 2017-08-28T21:16:40Z
Bron: https://wiki.osdev.org/User:Johnburger/Demo/Pad/ISO/Catalog
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJohnburger%2FDemo%2FPad%2FISO%2FCatalog&oldid=21377

````text
This Sector is where the "magic" happens. It defines the very first Sector on the CD as the default Boot Sector, and for the BIOS to treat the CD as a Floppy Disk.

It is only because [[ISO 9660]] treats the first 16 (2 kiB) Sectors as "reserved" that this trick works: since the BIOS looks for the Primary and Boot Volumes from the 17th Sector, we can put whatever we like before it:
* This Demonstrator;
* A complete OS;
* A loader for the rest of the CD, perhaps with an ISO driver;
* Anything! Over to you...

== <code>[[User:Johnburger/Demo/Demo|Demo]]/[[User:Johnburger/Demo/Overview#Padding|Pad]]/[[User:Johnburger/Demo/Pad/ISO/ISO|ISO]]/Catalog.inc</code> ==
<source lang="asm">
;
; Pad/ISO/Catalog.inc
;

; This file defines the Boot Catalog Sector for the ISO image. This basically
; points to the first sector, which is where the standard boot sector is anyway.
;
; Note that ISO.Boot.Default.Media has been set to Floppy.1440. This option
; works for VMware, VirtualPC and VirtualBox. Other options work for VMware only

%define         ISO.Catalog.ID          Version.Author

ISO.Sector.Catalog   EQU                ISO.Sectors
%assign              ISO.Sectors        ISO.Sectors + 1

ISO.Boot.Catalog:

; Add Validation entry, and calculate Checksum for us
ISO.Boot.Validation ISO.Validation      ISO.Catalog.ID, ISO.Boot.Platform.x86

ISO.Boot.Default:
.Bootable       DB              88h
.Media          DB              ISO.Boot.Media.Floppy.1440 ; HardDisk ; NoEmulation
.Load           DW              0000h   ; This could be 7C00h. 0000h defaults it
.System         DB              00h
                DB              0
.Count          DW              1       ; You can use this to load MORE!
.LBA            DD              0       ; Start sector to load
.Reserved       TIMES           (1Fh-0Ch+1) DB 00h

ISO.Boot.Catalog.Padding EQU    ISO.Sector.Size - ($ - ISO.Boot.Catalog)
                TIMES           ISO.Boot.Catalog.Padding DB 00h
</source>
````

<a id="w003588"></a>

## W003588 — User:Pancakes/arm qemu realview-pb-a

Bron-ID: W003588
Namespace: User (2)
Revisie: 16327 | Datum: 2014-05-01T00:04:19Z
Bron: https://wiki.osdev.org/User:Pancakes/arm_qemu_realview-pb-a
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2Farm_qemu_realview-pb-a&oldid=16327

````text
== ARM QEMU REALVIEW-PB-A ==
This is an ARM board emulated by QEMU that supports 4 cores. I hope to help ease you into
working with this board by giving you the information needed to at least get control over
what is going on.

See [[ARM_Overview]], (at the end) for links to tutorials, other boards, and stuff.

==== How To Use It ====
I just realized that someone who comes to this page might be lost as in how to use this board. You could of course find one and buy it, but if you did that you might find it easier and better just to 
buy some of the newer things out there.

So the main reason you would be interested in this board is for QEMU. Here is my boot line that
I am using and you will likely find pretty useful too. I do not use graphics because the darned
window pops up and annoys me because I am mainly looking at serial output. I start all 4 CPUs
and set the memory to 64MB. Also specify my flat binary image, and yes it can handle an ELF32 
image. It auto detects what type of image it is. The ''realview-pbx-a9'' is the actual board. By
default QEMU will normally use the ''integrator-cp'' which is a single core board, but it comes with
a graphics display that I have detailed how to initialize and draw pixels too somewhere around here.
<pre>
    qemu-system-arm -M realview-pbx-a9 -smp 4 -m 64 -kernel armos.bin -serial stdio -nographic
</pre>


==== Multiple Cores On Boot ====
From my testing it appears that the boot ROM (special boot ROM) grabs the other cores
and puts them into a WFI sleeping loop. They basically go to sleep until an interrupt 
wakes them then they check there special boot register (explained further below), and
if it is set to non-zero they jump to that address. One easy way to wake them is to 
send a SGI (software generated interrupt) to them. See the section below on waking up
the CPUs.

Below is an example of getting the ID of the CPU and placing it into a busy loop.
<source lang="c">
	/* stop all cpus but cpu 0 */
	asm("	mrc p15, 0, r0, c0, c0, 5\n\
		and r0, r0, #3\n\
		cmp r0, #0\n\
		____here: bne ____here\n"
	);
</source>

This will access the CPUID (for currently execting CPU), and if the ID is not zero then it
will be placed into a busy loop thus only allowing ''CPU0'' to continue.

''Also, at this point you could branch each CPU to it's own handler and set a stack for each
if desired.''

This will use the serial to output the letter 'A' for CPU0, 'B' for CPU1, 'C' for CPU2, and
'D' for CPU3. But, from my testing only CPU0 and CPU1 start on a cold boot at your entry 
address when loading a flat binary file (by default loaded to 0x10000).
<source lang="c">
	asm("	mrc p15, 0, r0, c0, c0, 5\n\
		and r0, r0, #3\n\
		mov r1, #0x1000\n\
		lsl r1, r1, #16\n\
		orr r1, r1, #0x9000\n\
		mov r2, #65\n\
		add r2, r2, r0\n\
		str r2, [r1]\n\
		cmp r0, #0\n\
		____here: bne ____here\n"
	);
</source>

A more appropriate way to handle trapping the CPUs might be to rather issue them a //WFI// or //WFE// instruction so they go into low-power mode while waiting for your main CPU to wake them up again with a SGI (software generated interrupt). This is the method that I currently use. Somewhere below you can find code to issue an SGI (its not an instruction).

==== Waking Up Other Cores ====
To wake up the other cores (with QEMU at least) this is the 
minimal process required:
<source lang="c">
#define REALVIEWPBA_PERBASE	0x1f000000	/* peripheal base */
#define REALVIEWPBA_GICOFF	0x0100	/* general interrupt controller */
#define REALVIEWPBA_GDIOFF	0x1000  /* GIC distributor */

	t = (uint32*)0x10000030;
	t[0] = 0x10000;
	
	/* enable general interrupt controller */
	b = (uint8*)(REALVIEWPBA_PERBASE + REALVIEWPBA_GICOFF);
	b[0] = 3;     /* enable */
	b[4] = 0xff;  /* set required priority */
	
	/* enable distributor then send SGI with distributor */
	t = (uint32*)(REALVIEWPBA_PERBASE + REALVIEWPBA_GDIOFF);
	t[0] = 3;     /* enable */
	
	t[0xf00 >> 2] = (1 << 24) | 0;    /* issue SGI 0 */
</source>
You should be able to use any SGI, which are 0-15. There are also other
things that will wake the CPUs up. The boot code places them to sleep with
the ''WFI'' (wait for interrupt) instruction. Once woken the address
''0x10000030'' is checked if it is non-zero. If it is zero it returns to
the ''WFI'' instruction and sleeps, but if it is non-zero it will jump to
the 32-bit address specified at ''0x10000030''.

==== Setting Up PIC And Timer ====
Here is some simple demonstration code to setup the PIC and PIT. For more
information refer to the datasheet for each device.  This PIT is global to all
CPUs so do not confuse it with the private PIT for each CPU (which is exampled further below).

''This has ONLY been tested on QEMU. On real hardware you might need to do more, and
you should refer to your datasheets on the hardware to diagnose problems. But, for QEMU
this code should work for release QEMU-2.0.0-rc0.''
<source lang="c">
#define CTRL_ENABLE			0x80
#define CTRL_MODE_FREE		0x00
#define CTRL_MODE_PERIODIC	0x40
#define CTRL_INT_ENABLE		(1<<5)
#define CTRL_DIV_NONE		0x00
#define CTRL_DIV_16			0x04
#define CTRL_DIV_256		0x08
#define CTRL_SIZE_32		0x02
#define CTRL_ONESHOT		0x01

#define REG_LOAD		0x00
#define REG_VALUE		0x01
#define REG_CTRL		0x02
#define REG_INTCLR		0x03
#define REG_INTSTAT		0x04
#define REG_INTMASK		0x05
#define REG_BGLOAD		0x06


        uint8   *picmmio1;
        uint8   *picmmio0;
        uint32  *pitmmio;
        uint32  *serialmmio;

        picmmio0 = (uint8*)0x1f000100;
        picmmio1 = (uint8*)0x1f001000;
        pit = (uint32*)0x10011000;
        serialmmio = (uint32*)0x10009000;
        

	/* talk to CPU interface*/
	/* enable PIC for CPU 0 */
	picmmio0[0] = 1;
	/* set priority mask for CPU 0 */
	picmmio0[4] = 0xff;

	/* talk to actual PIC stuff */
	/* enable PIC */
	picmmio1[0] = 1;
        /* 36 is the interrupt (on cold boot) for the timer below */	
	picmmio1[0x100 + (36 >> 3)] = (1 << (36 & 7));
	
	pitmmio[REG_LOAD] = KTASKTICKS;
	pitmmio[REG_BGLOAD] = KTASKTICKS;			
	pitmmio[REG_CTRL] = CTRL_ENABLE | CTRL_MODE_PERIODIC | CTRL_SIZE_32 | CTRL_DIV_NONE | CTRL_INT_ENABLE;
        pitmmio[REG_INTCLR] = ~0;		/* make sure interrupt is clear (might not be mandatory) */

        /* write character to serial output */
        serialmmio[0] = 'A';
</source>

==== GICC and GICD ====
The GICC is known as the ''CPU Interface'' and the GICD is known as the ''Distributor''. The GIC is for ''General Interrupt Controller'' and actually represents the two devices. The GICC is local to each CPU while the GICD sits in front of these controllers and distributes the interrupts to them.

Each CPU has it's on GICC mapped at the same address (not required by SoCs but recommend). So this makes it not possible for one CPU to access the GICC of another CPU unless they are mapped in different places and made accessible (not sure if possible). So just assume that you can only access the local GICC for with each core.

''Since yourprimary CPU can not access the GICC for the other cores the boot code (flashed into ROM) on the device will configure the GICC for each core therefore allowing you to send an SGI to wake them up.''

The GICC base is located at 0x1f000100, and the GICD base is located at 0x1f001000.

==== Peripheral Base ====

Here are just a few offsets to the CPU specific stuff, except the GICD which (AFAIK) is global.
<source lang="c">
#define REALVIEWPBA_PERBASE	0x1f000000	/* peripheal base */
#define REALVIEWPBA_SCUOFF	0x0000	/* snoop control unit */
#define REALVIEWPBA_GICOFF	0x0100	/* general interrupt controller */
#define REALVIEWPBA_GTIOFF	0x0200	/* global timer */
#define REALVIEWPBA_PTIOFF	0x0600	/* private timer */
#define REALVIEWPBA_GDIOFF	0x1000  /* GIC distributor */
</source>

==== Private Timer Per CPU (Initialization And Handling Example) ====

The ''PTIOFF'' is a private timer for each CPU. I can not remember if it is specified for the cortex-a9 or the realview-pb. I want to say the cortex-a9, but in any event. Each CPU has it's
own timer which is accessed as ''REALVIEWPBA_PERBASE + REALVIEWPBA_PTIOFF''. Here is some example code to enable and timer and handle its interrupt:

<source lang="c">
#define SET1BF(bf, off, i) bf[off + (i >> 3)] = (1 << (i & 7))

        uint8   *gicc, *gicd;
        uint32  *pt;

        gicc = (uint8*)(REALVIEWPBA_PERBASE + REALVIEWPBA_GICOFF);
        gicd = (uint8*)(REALVIEWPBA_PERBASE + REALVIEWPBA_GDIOFF);
	pt = (uint32*)(REALVIEWPBA_PERBASE + REALVIEWPBA_PTIMER);

	/* enable IRQ and FIQ OR just enable one */
	arm4_cpsrset(arm4_cpsrget() & ~((1 << 7) | (1 << 6)));
	
	/* enable local PIC */
	gicc[0x100 + 0] = 1;
	/* set priority mask*/
	gicc[0x100 + 4] = 0xff;

        /* enable global distributor */
	gicd[0] = 1;	
	
	
	/* enable interrupt 29 in distributor */
	SET1BF(gicd, 0x100, 29);

        /* enable timer */
	pt[2] = 0;
	pt[0] = 0x4000000;
	pt[2] = (1 << 2) | (1 << 1) | (1 << 0);
</source>

See the technical documents for exactly meaning of bits. To handle an interrupt you use
the following general process.

<source lang="c">

        uint8   *gicc;
        uint32  *pt;
        uint32  irq;

        gicc = (uint8*)(REALVIEWPBA_PERBASE + REALVIEWPBA_GICOFF);
	pt = (uint32*)(REALVIEWPBA_PERBASE + REALVIEWPBA_PTIMER);

        /* get interrupt to handle (irq == 1023 means spurious interrupt) */
        irq = gicc[3];
        if (irq == 29) {
                /* private timer is interrupt 29 (clear private timer interrupt line) */
                pt[3] = 1;
                /* not sure but you might have to clear interrupt below THEN clear private timer interrupt */
        }
        /* clear interrupt by saying we handled it */
        gicc[4] = irq;
</source>
The above is, of course, part of you execution chain from an IRQ/FIQ exception.

==== Hardware ====
This is the only datasheet that I have found, http://infocenter.arm.com/help/topic/com.arm.doc.dui0411d/DUI0411D_realview_platform_baseboard_ug.pdf. I am not sure how accurate it is to the revision the QEMU board was built too.

Here is a fairly accurate memory map of all the MMIO.
<pre>
    /* Memory map for RealView Emulation Baseboard:  */
    /* 0x10000000 System registers.  */
    /* 0x10001000 System controller.  */
    /* 0x10002000 Two-Wire Serial Bus.  */
    /* 0x10003000 Reserved.  */
    /* 0x10004000 AACI.  */
    /* 0x10005000 MCI.  */
    /* 0x10006000 KMI0.  */
    /* 0x10007000 KMI1.  */
    /* 0x10009000 UART0.  */
    /* 0x1000a000 UART1.  */
    /* 0x1000b000 UART2.  */
    /* 0x1000c000 UART3.  */
    /* 0x1000d000 SSPI.  */
    /* 0x1000e000 SCI.  */
    /* 0x1000f000 Reserved.  */
    /* 0x10010000 Watchdog.  */
    /* 0x10011000 Timer 0+1.  */
    /* 0x10012000 Timer 2+3.  */
    /* 0x10013000 GPIO 0.  */
    /* 0x10014000 GPIO 1.  */
    /* 0x10015000 GPIO 2.  */
    /* 0x10002000 Two-Wire Serial Bus - DVI. (PB) */
    /* 0x10017000 RTC.  */
    /* 0x10018000 DMC.  */
    /* 0x10019000 PCI controller config.  */
    /* 0x10020000 CLCD.  */
    /* 0x10030000 DMA Controller.  */
    /* 0x10080000 SMC.  */
    /* 0x1e000000 GIC1. (PB) */
    /* 0x1e001000 GIC2. (PB) */
    /* 0x1e002000 GIC3. (PB) */
    /* 0x1e003000 GIC4. (PB) */
    /* 0x40000000 NOR flash.  */
    /* 0x44000000 DoC flash.  */
    /* 0x48000000 SRAM.  */
    /* 0x4c000000 Configuration flash.  */
    /* 0x4e000000 Ethernet.  */
    /* 0x4f000000 USB.  */
    /* 0x50000000 PISMO.  */
    /* 0x54000000 PISMO.  */
    /* 0x58000000 PISMO.  */
    /* 0x5c000000 PISMO.  */
    /* 0x60000000 PCI.  */
    /* 0x60000000 PCI Self Config.  */
    /* 0x61000000 PCI Config.  */
    /* 0x62000000 PCI IO.  */
    /* 0x63000000 PCI mem 0.  */
    /* 0x64000000 PCI mem 1.  */
    /* 0x68000000 PCI mem 2.  */
</pre>
````

<a id="w003589"></a>

## W003589 — User:Pancakes/ARM QEMU REALVIEW-PB-A

Bron-ID: W003589
Namespace: User (2)
Revisie: 16212 | Datum: 2014-04-17T18:12:13Z
Bron: https://wiki.osdev.org/User:Pancakes/ARM_QEMU_REALVIEW-PB-A
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APancakes%2FARM_QEMU_REALVIEW-PB-A&oldid=16212
Doorverwijzing: User:Pancakes/arm qemu realview-pb-a

````text
#REDIRECT [[User:Pancakes/arm qemu realview-pb-a]]
````

<a id="w003590"></a>

## W003590 — Category:Multiprocessing

Bron-ID: W003590
Namespace: Category (14)
Revisie: 16224 | Datum: 2014-04-17T20:03:13Z
Bron: https://wiki.osdev.org/Category:Multiprocessing
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMultiprocessing&oldid=16224

````text
Multiprocessing is becoming more and more common. No longer is it NUMA servers in workplaces like Google, it now is SMP in your average $500 computer. Not to mention Hyperthreading. Multiprocessor can take two, software indivisble forms: seperate CPU dies or multi-core CPUs.

[[Category:OS theory]]
````

<a id="w003591"></a>

## W003591 — MP Specification

Bron-ID: W003591
Namespace: Main (0)
Revisie: 29078 | Datum: 2024-06-07T13:37:11Z
Bron: https://wiki.osdev.org/MP_Specification
Revisie-URL: https://wiki.osdev.org/index.php?title=MP_Specification&oldid=29078

````text
{{In Progress}}
==Introduction==
The MP specification is designed to make multiprocessing OEM independent. This allows OS support without an OEM driver. Therefore, it makes multiprocessing and all it's problems open to the hobbyist. it is aimed at X86. Please note that X86 multiprocessing is strictly SMP.

==Parts==
===MP Hardware===
MP is implemented as PC-AT compatible. One processor runs at boot time, the BSP (Boot System Processor). All other CPUs, APs (Application Processors), are disabled either by hardware or the BIOS (or both). The APs have to be enabled by the OS (hence AT compatibility), via APIC IPIs (INIT & STARTUP). Before enabling, they are in a halt state with interrupts disabled (however INIT & STARTUP IPIs are not masked).

===MP BIOS===
The BIOS has to construct MP tables based on the detected/known configuration. These are then used by a MP-capable OS.

===MP Tables===
====MP Floating Pointer====
====MP Configuration Table====

==See Also==
===On the Wiki===
*[[SMP]]
*[[Multiprocessing]]
===External Links===
*[http://www.intel.com/design/pentium/datashts/24201606.pdf MP Specification]

[[Category:X86]]
[[Category:Multiprocessing]]
[[Category:Standards]]
````

<a id="w003592"></a>

## W003592 — User:Max/Building a GCC 4.9 cross compiler on OS X Mavericks

Bron-ID: W003592
Namespace: User (2)
Revisie: 18219 | Datum: 2015-07-06T10:54:38Z
Bron: https://wiki.osdev.org/User:Max/Building_a_GCC_4.9_cross_compiler_on_OS_X_Mavericks
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMax%2FBuilding_a_GCC_4.9_cross_compiler_on_OS_X_Mavericks&oldid=18219

````text
In this tutorial you will learn how to build a GCC cross compiler toolchain on Mac OS X (10.9, Mavericks). This includes building cross binutils, a cross compiler and libgcc for your target architecture.

To do this tutorial, I assume that you have a basic knowledge about using the Terminal and commands like <tt>cd</tt> and <tt>mkdir</tt>.

== Installing GCC ==
To build a cross compiler from the GCC sources, you first need to have the GNU GCC installed on your system (called the "host system"). This compiler will later be used to compile the cross-toolchain.

There might already be a compiler showing up when using <tt>gcc</tt> in Terminal, but this is not the actual GNU GCC, it's Apples LLVM compiler. To install the real GNU GCC, we use [http://brew.sh/ Homebrew]. Once you have Homebrew installed, open up a Terminal. Then install GCC with the <tt>brew</tt> commands:

 brew tap homebrew/versions
 brew install --enable-cxx gcc49

This may take a while. It installs the GNU GCC 4.9 with additional support for C++. You should then check if GCC is working by using <tt>"gcc-4.9 --version"</tt> in the Terminal.

== Installing the necessary libraries ==
For building GCC you also need to install the libraries [https://gmplib.org/ GMP], [http://www.mpfr.org/ MPFR] and [http://www.multiprecision.org/ MPC] on your host system. You can do this with the following <tt>brew</tt> commands:

 brew install mpfr
 brew install gmp
 brew install libmpc

Once this is done, the necessary libraries are available on your system, and you are ready for building the cross compiler.

== Downloading the sources ==
The following sources are required to build the cross compiler toolchain. The versions here are the ones I'll be using in the following (and that are in the moment I'm writing this the newest sources). Any newer/older version might - or might not - work as well.

* [https://gnu.org/software/gcc/ GCC 4.9.0]
* [http://www.gnu.org/software/binutils/ binutils 2.24]

Once you've downloaded the archive files of the current sources (namely in our case <tt>gcc-4.9.0.tar.gz</tt> and <tt>binutils-2.24.tar.gz</tt>), create a folder in your home directory (name it for example <tt>src</tt>) and unpack the archives there.

== Building ==
Open up a terminal and <tt>cd</tt> to the <tt>src</tt> directory we just created. Inside of that directory, you should now have the folders <tt>gcc-4.9.0</tt> and <tt>binutils-2.24</tt> containing our necessary sources.

=== Preparing environment variables ===
We will now set some environment variables for the build process by using the <tt>export</tt> command. Take care - these variables are temporary, if you open a new terminal session, you'll have to enter them again.

First, we set the compiler variables right, to let the configure script and makefiles of binutils and gcc know that they need to use the compiler we just installed instead of the LLVM:

 export CC=gcc-4.9
 export CXX=g++-4.9
 export CPP=cpp-4.9
 export LD=gcc-4.9

Also, we define the following variables. The <tt>PREFIX</tt> is the path where you want to install the cross-binaries to. The <tt>TARGET</tt> determines your target platform, for example we want to build an <tt>i686-elf</tt> compiler. The <tt>PATH</tt> export we're doing here just appends the directory that you specified as the target for your cross-binaries to the <tt>PATH</tt>.

 export PREFIX="$HOME/opt/cross"
 export TARGET=i686-elf
 export PATH="$PREFIX/bin:$PATH"

Now we are ready to continue building binutils and then the gcc and libgcc.

=== binutils ===
<ol>
<li>Create a folder <tt>binutils-2.24-build</tt> in your <tt>~/src</tt> folder</li>
<li><tt>cd</tt> to that folder</li>
<li>From inside of that folder, we now run the configure script of binutils:</li>
 ../binutils-2.24/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --disable-werror
<li>Once the configure script has successfully completed its work, perform the build and installation:</li>
 make
 make install
</ol>

=== gcc and libgcc ===
<ol>
<li>Create a folder <tt>gcc-4.9.0-build</tt> in your <tt>~/src</tt> folder</li>
<li><tt>cd</tt> to that folder</li>
<li>From inside of that folder, we now run the configure script for GCC:<br/>
<small>(<b>Note:</b> the configure script might throw an error about not finding the installation of MPC/GMP/MPFR. To make it find these libraries, use the respective "--with-*" parameter. When using brew, this is by default "/usr/local", so "--with-gmp=/usr/local" etc. should work)</small></li>
 ../gcc-4.9.0/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers
<li>Once the configure script has successfully completed its work, perform the build and installation:</li>
 make all-gcc
 make all-target-libgcc
 make install-gcc
 make install-target-libgcc
</ol>

=== Checking the installation ===
You can now check if the cross compiler works by typing <tt>i686-elf-g++ --version</tt>. To make the new cross tools always available in Terminal, you will have to open your user <tt>.bash_profile</tt> like this:
 nano ~/.bash_profile

Now, add the following line (if you changed where you installed your crosstools to, you have to change it accordingly):
 export PATH=$HOME/opt/cross/bin:$PATH

Press <tt>^x</tt> for saving, press <tt>y</tt> and then <tt>enter</tt> to confirm saving. The crosstools are now in your PATH and are available in your Terminal.

==== Location of crosstools and target-libgcc ====

If you used the PREFIX variable from above, you should find a <tt>opt</tt> folder in your home directory. Inside of that folder is a <tt>cross</tt> folder, which contains all the stuff we just built. This folder contains:
* a <tt>bin</tt> folder with the binaries of the cross compiler
* a <tt>lib/gcc/i686-elf/4.9.0/</tt> folder which contains a <tt>include</tt> folder and <tt>libgcc.a</tt> for linking it with your kernel
````

<a id="w003593"></a>

## W003593 — User:Mrvn/LinkedListBucketHeapImplementation

Bron-ID: W003593
Namespace: User (2)
Revisie: 16346 | Datum: 2014-05-06T20:12:37Z
Bron: https://wiki.osdev.org/User:Mrvn/LinkedListBucketHeapImplementation
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMrvn%2FLinkedListBucketHeapImplementation&oldid=16346

````text
== Linked List Bucket Implementation ==
I prefixed all the external functions with ''mrvn'' the IRC name of the guy who wrote 
this implementation. You can find his email and name before at the top of the source.

You should be able to implement any of the standard C library calls with ones of your kernel
or your own user space. It has very minimal dependencies. 

=== Usage ===
<source lang="c">
    mrvn_memory = malloc(p);
    mrvn_memory_init(mrvn_memory, p);
    p = (uintptr)mrvn_malloc(p);
    mrvn_free((void*)p);
</source>

=== Source ===
<source lang="c">
/* 
	Linked List Bucket Heap 2013 Goswin von Brederlow <goswin-v-b@web.de>  
	
*/
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include "mrvn.h"

typedef struct DList DList;
struct DList {
    DList *next;
    DList *prev;
};

// initialize a one element DList
static inline void dlist_init(DList *dlist) {
	//printf("%s(%p)\n", __FUNCTION__, dlist);
    dlist->next = dlist;
    dlist->prev = dlist;
}

// insert d2 after d1
static inline void dlist_insert_after(DList *d1, DList *d2) {
	//printf("%s(%p, %p)\n", __FUNCTION__, d1, d2);
    DList *n1 = d1->next;
    DList *e2 = d2->prev;

    d1->next = d2;
    d2->prev = d1;
    e2->next = n1;
    n1->prev = e2;
}

// insert d2 before d1
static inline void dlist_insert_before(DList *d1, DList *d2) {
	//printf("%s(%p, %p)\n", __FUNCTION__, d1, d2);
    DList *e1 = d1->prev;
    DList *e2 = d2->prev;

    e1->next = d2;
    d2->prev = e1;
    e2->next = d1;
    d1->prev = e2;
}

// remove d from the list
static inline void dlist_remove(DList *d) {
	//printf("%s(%p)\n", __FUNCTION__, d);
    d->prev->next = d->next;
    d->next->prev = d->prev;
    d->next = d;
    d->prev = d;    
}

// push d2 to the front of the d1p list
static inline void dlist_push(DList **d1p, DList *d2) {
	//printf("%s(%p, %p)\n", __FUNCTION__, d1p, d2);
    if (*d1p != NULL) {
	dlist_insert_before(*d1p, d2);
    }
    *d1p = d2;
}

// pop the front of the dp list
static inline DList * dlist_pop(DList **dp) {
	//printf("%s(%p)\n", __FUNCTION__, dp);
    DList *d1 = *dp;
    DList *d2 = d1->next;
    dlist_remove(d1);
    if (d1 == d2) {
	*dp = NULL;
    } else {
	*dp = d2;
    }
    return d1;
}

// remove d2 from the list, advancing d1p if needed
static inline void dlist_remove_from(DList **d1p, DList *d2) {
	//printf("%s(%p, %p)\n", __FUNCTION__, d1p, d2);
    if (*d1p == d2) {
	dlist_pop(d1p);
    } else {
	dlist_remove(d2);
    }
}

#define CONTAINER(C, l, v) ((C*)(((char*)v) - (intptr_t)&(((C*)0)->l)))
#define OFFSETOF(TYPE, MEMBER)  __builtin_offsetof (TYPE, MEMBER)

#define DLIST_INIT(v, l) dlist_init(&v->l)

#define DLIST_REMOVE_FROM(h, d, l)					\
    {									\
	typeof(**h) **h_ = h, *d_ = d;					\
	DList *head = &(*h_)->l;					\
	dlist_remove_from(&head, &d_->l);					\
	if (head == NULL) {						\
	    *h_ = NULL;							\
	} else {							\
	    *h_ = CONTAINER(typeof(**h), l, head);			\
	}								\
    }

#define DLIST_PUSH(h, v, l)						\
    {									\
	typeof(*v) **h_ = h, *v_ = v;					\
	DList *head = &(*h_)->l;					\
	if (*h_ == NULL) head = NULL;					\
	dlist_push(&head, &v_->l);					\
	*h_ = CONTAINER(typeof(*v), l, head);				\
    }

#define DLIST_POP(h, l)							\
    ({									\
	typeof(**h) **h_ = h;						\
	DList *head = &(*h_)->l;					\
	DList *res = dlist_pop(&head);					\
	if (head == NULL) {						\
	    *h_ = NULL;							\
	} else {							\
	    *h_ = CONTAINER(typeof(**h), l, head);			\
	}								\
	CONTAINER(typeof(**h), l, res);					\
    })

#define DLIST_ITERATOR_BEGIN(h, l, it)					\
    {									\
        typeof(*h) *h_ = h;						\
	DList *last_##it = h_->l.prev, *iter_##it = &h_->l, *next_##it;	\
	do {								\
	    if (iter_##it == last_##it) {				\
		next_##it = NULL;					\
	    } else {							\
		next_##it = iter_##it->next;				\
	    }								\
	    typeof(*h)* it = CONTAINER(typeof(*h), l, iter_##it);

#define DLIST_ITERATOR_END(it)						\
	} while((iter_##it = next_##it));				\
    }

#define DLIST_ITERATOR_REMOVE_FROM(h, it, l) DLIST_REMOVE_FROM(h, iter_##it, l)

typedef struct Chunk Chunk;
struct Chunk {
    DList all;
    int used;
    union {
	char data[0];
	DList free;
    };
};

enum {
    NUM_SIZES = 32,
    ALIGN = 4,
    MIN_SIZE = sizeof(DList),
    HEADER_SIZE = OFFSETOF(Chunk, data),
};

Chunk *free_chunk[NUM_SIZES] = { NULL };
size_t mem_free = 0;
size_t mem_used = 0;
size_t mem_meta = 0;
Chunk *first = NULL;
Chunk *last = NULL;

static void memory_chunk_init(Chunk *chunk) {
	//printf("%s(%p)\n", __FUNCTION__, chunk);
    DLIST_INIT(chunk, all);
    chunk->used = 0;
    DLIST_INIT(chunk, free);
}

static size_t memory_chunk_size(const Chunk *chunk) {
	//printf("%s(%p)\n", __FUNCTION__, chunk);
    char *end = (char*)(chunk->all.next);
    char *start = (char*)(&chunk->all);
    return (end - start) - HEADER_SIZE;
}

static int memory_chunk_slot(size_t size) {
    int n = -1;
    while(size > 0) {
	++n;
	size /= 2;
    }
    return n;
}

void mrvn_memory_init(void *mem, size_t size) {
    char *mem_start = (char *)(((intptr_t)mem + ALIGN - 1) & (~(ALIGN - 1)));
    char *mem_end = (char *)(((intptr_t)mem + size) & (~(ALIGN - 1)));
    first = (Chunk*)mem_start;
    Chunk *second = first + 1;
    last = ((Chunk*)mem_end) - 1;
    memory_chunk_init(first);
    memory_chunk_init(second);
    memory_chunk_init(last);
    dlist_insert_after(&first->all, &second->all);
    dlist_insert_after(&second->all, &last->all);
    // make first/last as used so they never get merged
    first->used = 1;
    last->used = 1;

    size_t len = memory_chunk_size(second);
    int n = memory_chunk_slot(len);
    //printf("%s(%p, %#lx) : adding chunk %#lx [%d]\n", __FUNCTION__, mem, size, len, n);
    DLIST_PUSH(&free_chunk[n], second, free);
    mem_free = len - HEADER_SIZE;
    mem_meta = sizeof(Chunk) * 2 + HEADER_SIZE;
}

void *mrvn_malloc(size_t size) {
    //printf("%s(%#lx)\n", __FUNCTION__, size);
    size = (size + ALIGN - 1) & (~(ALIGN - 1));
    
	if (size < MIN_SIZE) size = MIN_SIZE;
    
	int n = memory_chunk_slot(size - 1) + 1;
    
	if (n >= NUM_SIZES) return NULL;
    
	while(!free_chunk[n]) {
		++n;
		if (n >= NUM_SIZES) return NULL;
    }
    
	Chunk *chunk = DLIST_POP(&free_chunk[n], free);
    size_t size2 = memory_chunk_size(chunk);
	//printf("@ %p [%#lx]\n", chunk, size2);
    size_t len = 0;
    
	if (size + sizeof(Chunk) <= size2) {
		Chunk *chunk2 = (Chunk*)(((char*)chunk) + HEADER_SIZE + size);
		memory_chunk_init(chunk2);
		dlist_insert_after(&chunk->all, &chunk2->all);
		len = memory_chunk_size(chunk2);
		int n = memory_chunk_slot(len);
		//printf("  adding chunk @ %p %#lx [%d]\n", chunk2, len, n);
		DLIST_PUSH(&free_chunk[n], chunk2, free);
		mem_meta += HEADER_SIZE;
		mem_free += len - HEADER_SIZE;
    }

	chunk->used = 1;
    //memset(chunk->data, 0xAA, size);
	//printf("AAAA\n");
    mem_free -= size2;
    mem_used += size2 - len - HEADER_SIZE;
    //printf("  = %p [%p]\n", chunk->data, chunk);
    return chunk->data;
}

static void remove_free(Chunk *chunk) {
    size_t len = memory_chunk_size(chunk);
    int n = memory_chunk_slot(len);
    //printf("%s(%p) : removing chunk %#lx [%d]\n", __FUNCTION__, chunk, len, n);
    DLIST_REMOVE_FROM(&free_chunk[n], chunk, free);
    mem_free -= len - HEADER_SIZE;
}

static void push_free(Chunk *chunk) {
    size_t len = memory_chunk_size(chunk);
    int n = memory_chunk_slot(len);
    //printf("%s(%p) : adding chunk %#lx [%d]\n", __FUNCTION__, chunk, len, n);
    DLIST_PUSH(&free_chunk[n], chunk, free);
    mem_free += len - HEADER_SIZE;
}

void mrvn_free(void *mem) {
    Chunk *chunk = (Chunk*)((char*)mem - HEADER_SIZE);
    Chunk *next = CONTAINER(Chunk, all, chunk->all.next);
    Chunk *prev = CONTAINER(Chunk, all, chunk->all.prev);
    
	//printf("%s(%p): @%p %#lx [%d]\n", __FUNCTION__, mem, chunk, memory_chunk_size(chunk), memory_chunk_slot(memory_chunk_size(chunk)));
    mem_used -= memory_chunk_size(chunk);
	
    if (next->used == 0) {
		// merge in next
		remove_free(next);
		dlist_remove(&next->all);
		//memset(next, 0xDD, sizeof(Chunk));
		mem_meta -= HEADER_SIZE;
		mem_free += HEADER_SIZE;
    }
    if (prev->used == 0) {
		// merge to prev
		remove_free(prev);
		dlist_remove(&chunk->all);
		//memset(chunk, 0xDD, sizeof(Chunk));
		push_free(prev);
		mem_meta -= HEADER_SIZE;
		mem_free += HEADER_SIZE;
    } else {
		// make chunk as free
		chunk->used = 0;
		DLIST_INIT(chunk, free);
		push_free(chunk);
    }
}

#define MEM_SIZE (1024*1024*256)
char MEM[MEM_SIZE] = { 0 };

#define MAX_BLOCK (1024*1024)
#define NUM_SLOTS 1024
void *slot[NUM_SLOTS] = { NULL };

void check(void) {
	int	i;
    Chunk *t = last;
    
	DLIST_ITERATOR_BEGIN(first, all, it) {
		assert(CONTAINER(Chunk, all, it->all.prev) == t);
		t = it;
    } DLIST_ITERATOR_END(it);
	
    for(i = 0; i < NUM_SIZES; ++i) {
		if (free_chunk[i]) {
			t = CONTAINER(Chunk, free, free_chunk[i]->free.prev);
			DLIST_ITERATOR_BEGIN(free_chunk[i], free, it) {
			assert(CONTAINER(Chunk, free, it->free.prev) == t);
			t = it;
			} DLIST_ITERATOR_END(it);
		}
    }
}
</source>
````

<a id="w003594"></a>

## W003594 — Compiler

Bron-ID: W003594
Namespace: Main (0)
Revisie: 23942 | Datum: 2019-09-22T16:44:18Z
Bron: https://wiki.osdev.org/Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Compiler&oldid=23942

````text
A compiler is a program that translates code from a source programming language into target programming language. Usually the target programming language is of a lower level than the source programming language - compilers target languages such as C, byte code, assembly, or raw machine code.

A popular reason for languages to target another high-level language such as C is that compilers can focus purely on translating the language, while getting all of the world-class optimizations of a production quality C compiler. This process is much more difficult for dynamic languages.

Most compilers output assembly of some form or another, and pass it to an assembler. More rarely, some compilers may directly generate byte code or machine code without a third party assembler. (JIT compilers are a special class of embedded compilers that directly generate machine code.)

== Other tools ==
Here are some other common tools
=== Assemblers ===
A compiler takes a high level code that is often severely abstracted away from the underlying machine instructions that will eventually execute.

An assembler takes assembly code and outputs machine code. Assembly code is a one-to-one representation of machine instructions. Often the only transformation an assembler does is fill in labels (or keep a list of 'fixup' positions - which are filled in when multiple objects are linked together). More advanced assemblers may have macros or some kind of preprocessor, but often these macros generate machine instructions, so there is no abstraction away from the underlying instructions as a high level language could.

The assembler can either generate a executable binary file (perfect for running) or an object file. An object file is different to an executable binary file, as it usually includes a list of exported symbols and/or fixup addresses. Object files may either referred to files that can be loaded dynamically at runtime (dynamic libraries) or files that are intended to be merges with other object files, to share their labels and to produce the final binary file.

=== Linkers ===
A linker takes multiple object files, and combines them into either a single object file or the final binary file.

Linkers are often used when you want to link your code with a library, or when your program has multiple source files, each compiled individually, and you want to produce one binary file.

Each object file contains a list exported symbols and where they are located, and a list of 'fixup' positions and the symbol it's looking for. The linker then attempts to fill in the fixup positions with symbols from other files.

== Stages ==
Compilers have multiple stages. Each stage takes a data structure created by the previous stage and transforms into a new data structure. Advance compilers that support multiple input languages or multiple output languages are usually divided into a front-end, middle-end, and back-end - where the middle and back ends can be swapped. Simpler compilers that only support one input language and one target language often do not bother to separate these ends.

=== Front End ===
The front end of a compiler refers to the part of the compiler that reads the input language. In large compilers like [[GCC]] and [[LLVM]] that support multiple languages, the front ends and back ends can be mixed and matched to support different languages while sharing the same middle end.

==== Lexical Analysis ====
[[Lexical Analyzer |Lexical analysis]] reads the source file and splits it into tokens. A token usually has a type, which may be a symbol such as FOR, INT, VOID. A token may also be a literal such as STRING_LITERAL, IDENTIFIER, FLOAT_LITERAL with extra data (such as the value of the literal or name of the identifier) attached. The lexer usually handles skipping over comments.

Lexers can either be written by hand or they can be generated with a 'lexer generator' - which is a tool that takes in a lexical grammar and generates the code of a lexer.

==== Parsing ====
The parsing stage reads the tokens from the lexer, and builds some form of representation of the code in memory. The representation is usually an abstract syntax tree.

A parser can either be written by hand (research [http://en.wikipedia.org/wiki/Recursive_descent_parser Recursive descent parser]) or generated with a 'parser generator' - which is a tool that takes in a language grammar and generates the code of a parser. It is during this stages that local variables and object names are registered as they are encountered.

Some parser generators generate both parsers and lexers in one go and are called 'compiler compilers'.

====Abstract syntax tree====
An abstract syntax tree (AST) is the representation of a program in tree format, and is usually the representation of the source code in tree format. The abstract syntax tree is usually closely tied to the input language and hence part of front-end, but sometimes it may be part of the middle-end. The abstract syntax tree is then converted into a language-independent intermediate representation.

====Semantical graph====
Programming languages use identifiers to reference entities declared somewhere else. Before performing further analysis, identifiers should be resolved - connected to the declarations they mean. This turns AST into a graph. Graph is built using auxiliary data structure called symbol table or context.

In some compilers the same data structure is used for AST and semantical graph. In this case parser may set declaration pointers to NULL or identifier resolution may be integrated into parsing process.

====Semantical analysis====
This stage calculates attributes of expressions and declarations and performs all kinds of validity checks: determine types of the expressions, resolves function overloads, checks covariance of the overridden functions, checks that constant variables are not mutated, checks that all enum elements are handled in the switch statement, etc.

===Middle end===
The middle end acts as glue between the front and back ends. This is in an [http://en.wikipedia.org/wiki/Intermediate_representation#Intermediate_representation intermediate representation] that is language-agnostic, and most optimizations are done here. Not all compilers have middle ends, sometimes abstract syntax trees directly produce machine code and skip through these steps.

====Bytecode====
Some compilers have an internal bytecode language (which can be different from bytecode targets) for communicating between front ends and back ends. These bytecodes may either be register-based or stack based (stack based bytecode is [http://en.wikipedia.org/wiki/Stack_machine#Simple_compilers incredibly easy to generate from an abstract syntax tree]).

====Single static assignment====
The abstract syntax tree or bytecode may be translated into [http://en.wikipedia.org/wiki/Static_single_assignment_form single static assignment], or SSA, form. SSA form is an intermediate representation of code where all variables are only assigned to once. SSA form is great for optimizations such as constant folding and dead code elimination. It is also an easy form that maps nicely to registers in the backend.

===Back end===
The back end of a compiler outputs the code into the target language. Backends often perform target-specific optimizations.

====Interpreters====
Interpreters do not output code so they are technically not compilers, but do implement their front and middle ends similarly to compilers. They may execute the immediate representation, the abstract syntax tree, or as the language is parsing - bypassing a middle end completely. JIT (just-in-time) compilers generate machine code in memory and execute it immediately.

====Bytecode====
A compiler may target bytecode, like [[WebAssembly]]. Bytecode is designed to be executed inside of a virtual machine, and is often simpler to target (due to not having to allocate a limited number of registers, and since it's higher level) than assembly. 

====Assembly====
Compilers may target assembly language. This is similar to targeting bytecode, except the compiler must allocate registers, and often deal with register and stack allocations, and [http://en.wikipedia.org/wiki/Calling_convention calling conventions].

By outputing assembly language, the compiler leaves it up to the assembler to implement the object and executable formats, and to calculate code positions, labels, and encode the instructions in machine code.

====Machine code====
Compilers may generate machine code (such as JIT compilers). This is often done when the compiler has all of the input code needed and does not need the output file to be merged with other output files. The process for outputting machine code is very similar to outputting assembly code, but the compiler must now resolve label positions, encode machine instructions, and deal with outputting in an [[Executable Formats|executable format]].

[[Category:Compilers]]
````

<a id="w003595"></a>

## W003595 — User:No92

Bron-ID: W003595
Namespace: User (2)
Revisie: 20773 | Datum: 2017-05-07T09:57:16Z
Bron: https://wiki.osdev.org/User:No92
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANo92&oldid=20773

````text
Hi, I'm no92 from Germany.

== Contributions ==
* [[User:No92/ARM_Instruction_Encoding|ARM instruction encoding]] (incomplete)
* [[User:No92/Random_Bits|random notes]], mostly on tools
* various bits and pieces all over the wiki
````

<a id="w003597"></a>

## W003597 — AS86

Bron-ID: W003597
Namespace: Main (0)
Revisie: 19822 | Datum: 2016-09-17T15:46:46Z
Bron: https://wiki.osdev.org/AS86
Revisie-URL: https://wiki.osdev.org/index.php?title=AS86&oldid=19822

````text
{{Stub}}

AS86 is a legacy assembler, once used to build ancient versions of Linux until [[GAS]] took over. It's based on Intel syntax rather than the AT&T syntax more common amongst Unix-based assemblers. Despite being less well known, it is very similar to the more common assemblers in terms of its functionality. It differs in that there is little requirement for so-called "red tape" at the start of a source file, although there is limited support for macros (most notably jmp $ fails). 

The assembler often comes bundled with the rest of the bin86 toolchain, but as it is unmaintained and slowly disappearing it is recommended to use one of the more current Intel assemblers for new projects. Common alternatives are [[YASM]], [[NASM]] or [[FASM]].

== Usage ==
AS86 allows the user to limit the available instructions to those available on a specific processor generation (e.g. 286, 386, etc.). One can also choose to assemble the standard input, which can be useful for automated build scripts.

It is very easy to prepare a flat binary using AS86 with the help of [[LD]], as one can simply run "ld86 -d" against the produced object file. Using this method it is also possible to link object files in various formats.

== See Also ==
=== Articles ===
* [[GAS]]
* [[Assembly]]
* [[Tool Comparison]]
* [[MASM]]
* [[Wikibooks:X86 Assembly/MASM Syntax|MASM Syntax]] - Wikibook

[[Category:Assemblers]]
````

<a id="w003598"></a>

## W003598 — User:Madanra/Tasks before leaving Real Mode

Bron-ID: W003598
Namespace: User (2)
Revisie: 16430 | Datum: 2014-05-31T16:49:12Z
Bron: https://wiki.osdev.org/User:Madanra/Tasks_before_leaving_Real_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMadanra%2FTasks_before_leaving_Real_Mode&oldid=16430

````text
Before switching to [[Protected Mode]] or [[Long Mode]], there's a number of tasks you'll want your bootloader to perform that are hard or impossible afterwards.

== Tasks ==

*; Get a map of the physical address space
: The only safe and reliable way of getting a memory map is BIOS function INT 0x15, EAX=0xE820 or UEFI function GetMemoryMap. See [[Detecting Memory (x86)]] and [[Memory Map (x86)]].

*; Video tasks
: If you don't have a video driver for every graphics card in existence, you're going to need to use BIOS or UEFI functions. You may want to:

:*; Get a list of video modes

:*; Get the monitor's EDID

:*; Set the video mode

: See [[Getting VBE Mode Info]].

*; Get a list of disk drives present and their "virtual cylinders, heads, and sectors" information
: The CHS information is useful for correctly setting the CHS values in partition tables without guesswork, and you'll to know the mapping from "BIOS device number" to physical device so that after booting you know which devices the CHS geometry you found applies to.

*; If you booted from disk, find out what disk you booted from
: This is somewhat covered by the previous item, but is one its most important advantages: if you've loaded your kernel etc. using BIOS functions and a "BIOS device number" you'll need to know what physical device this corresponds to so you can identify what physical partition you booted from once your OS is running.

*; If you booted from network, get network information
: Find the network card's MAC address and IP address, plus the DNS server's IP address, TFTP server's IP address etc. so that you can avoid doing (relatively slow) DHCP negotiations a second time if/when possible.

*; BIOS only

:*; Determinte PCI access mechanism
:: It's best to ask the BIOS to avoid manual probing. See [[PCI]].

:*; Enable A20 line
:: It's best to ask BIOS to do it and fall back to less reliable methods if the BIOS won't. See [[A20 Line]]

:*; Tell the BIOS what mode you're going to use
:: For x86-64 processors, the AMD manuals specify a BIOS function INT 0x15, EAX=0xEC00, BL=mode which tells the BIOS whether we are planning to use protected mode only, long mode only, or a mixture.

*; UEFI only

:*; Get addresses of ACPI, SMBIOS etc. tables
:: UEFI tells you the location of various significat tables so you don't have to search for them.

*; Load your kernel/initrd/whatever
: Self-explanatory :)

== See also ==

=== Articles ===

* [[Journey To The Protected Land]]

=== Forum threads ===

* [http://forum.osdev.org/viewtopic.php?p=236336#p236336 Forum post that this is based on]
* [http://forum.osdev.org/viewtopic.php?p=161572#p161572 Forum post about INT 0x15, EAX=0xEC00]

=== External links ===
````

<a id="w003599"></a>

## W003599 — User talk:Habs

Bron-ID: W003599
Namespace: User talk (3)
Revisie: 16454 | Datum: 2014-06-18T14:22:20Z
Bron: https://wiki.osdev.org/User_talk:Habs
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AHabs&oldid=16454

````text
Hey habs, I sent you a PM on the forums about a recent wiki edit. You don't seem a forum regular so alerted you here as well. --[[User:Sortie|Sortie]] 09:22, 18 June 2014 (CDT)
````

<a id="w003600"></a>

## W003600 — User:ChosenOreo

Bron-ID: W003600
Namespace: User (2)
Revisie: 16492 | Datum: 2014-06-21T06:47:59Z
Bron: https://wiki.osdev.org/User:ChosenOreo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenOreo&oldid=16492

````text
== Introduction ==
Hi there! My name is Adrian Collado and I'm a student studying Electrical Engineering. Ever since I wrote my first program, I've been interested in how things work at a lower level (hence going into the EE field). One way I've pursued this interest is by attempting to develop my own Operating System, Tachyon (current incarnation of the OS). Ever since I began development, I've learned quite a bit about computers at a hardware level. The links on this page go to various bits of information about my operating system, hardware knowledge that I've gained, and projects I feel are rather cool.

== Links ==
=== Helpful Operating System Stuff ===
==== [[User:ChosenOreo/CPU Detection|Wiki Page: CPU Detection]] ====

==== [[User:ChosenOreo/Chipsets|Wiki Page: Chipsets]] ====
````

<a id="w003601"></a>

## W003601 — User:Chosenoreo

Bron-ID: W003601
Namespace: User (2)
Revisie: 19369 | Datum: 2016-06-19T14:53:24Z
Bron: https://wiki.osdev.org/User:Chosenoreo
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenoreo&oldid=19369
Doorverwijzing: User:ChosenOreo

````text
#REDIRECT [[User:ChosenOreo]]

== Introduction ==
Hi there! My name is Adrian Collado and I'm a student studying Electrical Engineering. Ever since I wrote my first program, I've been interested in how things work at a lower level (hence going into the EE field). One way I've pursued this interest is by attempting to develop my own Operating System, Tachyon (current incarnation of the OS). Ever since I began development, I've learned quite a bit about computers at a hardware level. The links on this page go to various bits of information about my operating system, hardware knowledge that I've gained, and projects I feel are rather cool.

== Links ==
=== Helpful Operating System Stuff ===
[[User:ChosenOreo/CPU Detection|CPU Detection]]
````

<a id="w003602"></a>

## W003602 — User:ChosenOreo/CPU Detection

Bron-ID: W003602
Namespace: User (2)
Revisie: 16486 | Datum: 2014-06-21T06:31:32Z
Bron: https://wiki.osdev.org/User:ChosenOreo/CPU_Detection
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenOreo%2FCPU_Detection&oldid=16486

````text
== Introduction ==
Since I first began developing my Operating System, I was concerned with not accurately detecting the system that the user is running on. I also was concerned with supporting the most hardware that I could. Thus, I went out in search of the best ways to detect various CPUs. These algorithms are documented here in this article, along with a brief description on where I got the information.

== x86 ==
Detecting the various kinds of CPUs in the x86 line can be very difficult at times, as there are an incredible selection of different CPUs, and there are some with very little differences or documentation.

The information in this section was obtained from a variety of sources. The first source is from Robert Collins, in his article [http://www.rcollins.org/ddj/Nov96/Nov96.html CPUID Algorithm Wars].

=== Detecting 8086/88 and 80186/88 vs 80286+ ===
There are many different ways to detect an 80286 and higher versus any CPUs that came before it. The two methods I've used are the stack pointer microcode bug method and the flags method.

==== Stack Pointer Microcode Bug ====
On the 8086/88 and 80186/88 CPUs, on a <code>PUSH SP</code> operation, the stack pointer was modified before it was pushed to the stack. Thus, if you popped that value from the stack into another register and then compared it to the current stack pointer value, on a 80286+ they would be the same and on anything prior to it they would be different.

Here is some example code:
<source lang="asm">; In this code, if the CPU is determined to be an 8086/88 or 80186/88, the zero flag will be set.
push sp
pop bx
cmp bx, sp
jz Cpu_Is_8086_Or_80186</source>

==== Flags Method ====
On the 8086/88 and 80186/88 CPUs, the flags register's bits 12-15 will always be set to 1. Thus, if we try to modify these values and they can be changed, then we must not have an 8086/88 or 80186/88.

Here is some example code:
<source lang="asm">; In this code, if the CPU is determined to be an 8086/88 or 80186/88, the zero flag will be set.
pushf
pop bx
mov ax, bx
and bh, 0x0f
push bx
popf
pushf
pop bx
and bh, 0xf0
cmp bh, 0xf0
push ax
popf
jz Cpu_Is_8086_Or_80186</source>

=== 8086/88 vs 80186/88 ===
The 8086/88 and 80186/88 processors were very similar except for a few opcodes which can't be tested unless we have an invalid opcode handler (which is not present on the 8086/88 CPUs). Thus, we have to determine whether we have an 8086/88 or an 80186/88 a different way. We can do this by pushing a word at offset 0xffff in a segment. On an 8086/88, the bytes of this word will end up at offsets 0xffff and 0x0000, while on an 80186 the word will end up at bytes 0xffff and 0x10000.

Here is some example code:
<source lang="asm">; In this code, if the CPU is determined to be an 8086/88, the zero flag will be set.
mov ax, ds:[0x0000]
mov bx, ds:[0xffff]
mov byte ds:[0x0000], 0x00
mov word ds:[0xffff], 0xffff
cmp byte ds:[0x0000], 0xff
mov ds:[0x0000], ax
mov ds:[0xffff], bx
jz Cpu_Is_8086_Or_8088</source>

=== x86 vs x88 ===
On both the 8086 and 80186, the prefetch queue is 6 bytes long. On the 8088 and 80188, it is only 4 bytes long. Thus, if we modify a value 5 bytes ahead of our current location, only the 8088 or 80188 processors should detect it.

Here is some example code:
<source lang="asm">; In this code, if the CPU is determined to be an x88, the zero flag will be set.
mov ax, cs
mov es, ax
std
mov dx, 0x0001
mov di, WriteLocation
mov al, 0x90
mov cx, 0x0003
rep stosb
cld
nop
nop
nop
dec dx
nop
WriteLocation:
nop
test dx, dx
jz Cpu_Is_x88</source>

=== Intel 8086/88 and 80186/88 vs NEC V20/V30 and V40/V50 ===
'''This is questioned!''' - On the Intel processors, during a multiply operation, the zero flag is always cleared. On the NEC processors, the zero flag is set according to the result. Thus, to test for the NEC, we simply multiply something by zero and then check the zero flag.

Here is some example code:
<source lang="asm">; In this code, if the CPU is determined to be a NEC CPU, the zero flag will be set.
xor al, al
mov al, 0x40
mul al
jz Cpu_Is_NEC
</source>

== See Also ==
=== Forum Topics ===
[http://forum.osdev.org/viewtopic.php?f=13&t=28192 Forum Post for this page]

=== Articles ===
[http://www.rcollins.org/ddj/Nov96/Nov96.html Great Article by Robert Collins]
````

<a id="w003603"></a>

## W003603 — User:ChosenOreo/Templates

Bron-ID: W003603
Namespace: User (2)
Revisie: 16494 | Datum: 2014-06-21T07:09:12Z
Bron: https://wiki.osdev.org/User:ChosenOreo/Templates
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenOreo%2FTemplates&oldid=16494

````text
The templates for the wiki pages I've written reside here.

[[User:ChosenOreo/Templates/Infobox Chipset|Chipset Infobox Template]]
````

<a id="w003604"></a>

## W003604 — User:ChosenOreo/Templates/Infobox Chipset

Bron-ID: W003604
Namespace: User (2)
Revisie: 16485 | Datum: 2014-06-21T06:08:33Z
Bron: https://wiki.osdev.org/User:ChosenOreo/Templates/Infobox_Chipset
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenOreo%2FTemplates%2FInfobox_Chipset&oldid=16485

````text
<table style="font-size:95%; line-height:1.5em; padding:0.25em; float:right; margin: 0 0 8px 15px; clear:right; border:1px solid #aaa; background:#ececec; text-align:center;{{{1|}}};"><!--

--><caption style="padding:0.3em; font-size:1.1em; display:table-caption">{{{name}}}</caption><!--

--><tr style="text-align:left; vertical-align:top;"><td style="padding:0 50px 0 0;">Type:</td><td>{{{type}}}</td></tr><!--

--><tr style="text-align:left;"><td style="padding:0 50px 0 0;">Manufacturer:</td><td>{{{manufacturer}}}</td></tr><!--

--><tr style="text-align:left; vertical-align:top;"><td style="padding:0 50px 0 0;">CPUs Supported:</td><td>{{{cpu}}}</td></tr><!--

--><tr style="text-align:left; vertical-align:top;"><td style="padding:0 50px 0 0;">Socket:</td><td>{{{socket}}}</td></tr><!--

--><tr style="text-align:left; vertical-align:top;"><td style="padding:0 50px 0 0;">Southbridge(s):</td><td>{{{southbridge}}}</td></tr><!--

--><!--{{Template:If|{{{northbridge|}}}|<tr style="text-align:left; vertical-align:top;"><td style="padding:0 50px 0 0;">Northbridge:</td><td>{{{northbridge}}}</td></tr>|}}--><!--

--><tr style="text-align:left; vertical-align:top;"><td style="padding:0 50px 0 0;">Release Date:</td><td>{{{date}}}</td></tr><!--

--></table>
````

<a id="w003605"></a>

## W003605 — User:ChosenOreo/Chipsets/Intel X58

Bron-ID: W003605
Namespace: User (2)
Revisie: 16488 | Datum: 2014-06-21T06:42:14Z
Bron: https://wiki.osdev.org/User:ChosenOreo/Chipsets/Intel_X58
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenOreo%2FChipsets%2FIntel_X58&oldid=16488

````text
{{User:ChosenOreo/Templates/Infobox Chipset
| name = X58
| type = Northbridge (I/O Hub)
| manufacturer = Intel
| cpu = Core i7<br />Xeon 5500 Series<br />Xeon 6500/7500 Series
| socket = LGA 1366
| southbridge = [[User:ChosenOreo/Chipsets/Intel ICH10|ICH10]]
| date = November 2008
}}

The Intel X58 Northbridge is an Intel chip designed to connect Intel CPUs to peripheral devices using Intel's point-to-point processor interconnect interface, Intel QuickPath Interconnect (QPI). The processors that support this chipset use the Nehalem microarchitecture, which means that the X58 does not have a memory interface (Nehalem-based processors have an integrated memory controller). The first processors to support this chipset were Core i7 processors, but the Xeon line of processors was soon included.

== Description ==
The Intel QuickPath Interconnect architecture is radically different from earlier chipsets produced by Intel. At a lower level, this chipset is closer to AMD's HyperTransport architecture. The X58 has most common properties of a typical northbridge, except for the lack of a memory interface. It communicates with the [[User:ChosenOreo/Chipsets/Intel ICH10|ICH10]] southbridge via Direct Media Interface, with the processor over the QuickPath Interconnect, and connect with high-bandwith peripherals via [[PCI Express]]. Since the X58 does not contain a memory interface, Intel decided to call it an I/O Hub (not to be confused with the southbridge, now called the Legacy I/O Controller Hub).

== Using the Chipset ==
TODO: Add Details on interfacing with the chipset.
== References ==
=== Articles ===
[http://en.wikipedia.org/wiki/Intel_X58 Wikipedia Article on Intel X58]<br />
[http://ark.intel.com/products/36785/Intel-X58-Express-IO-Hub Intel Specifications Quick List]
=== Documents ===
[http://www.intel.com/content/dam/doc/datasheet/x58-express-chipset-datasheet.pdf X58 Express Chipset Datasheet (pdf)]
````

<a id="w003606"></a>

## W003606 — User talk:ChosenOreo/Chipsets/Intel X58

Bron-ID: W003606
Namespace: User talk (3)
Revisie: 16489 | Datum: 2014-06-21T06:45:00Z
Bron: https://wiki.osdev.org/User_talk:ChosenOreo/Chipsets/Intel_X58
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AChosenOreo%2FChipsets%2FIntel_X58&oldid=16489

````text
I'm not sure about posting information on interfacing with the chipset. Am I allowed to post such information if I initially learned about it by reading the Intel documentation (since it is public documentation)? [[User:Chosenoreo|- ChosenOreo (acollado@citlink.net)]] 01:45, 21 June 2014 (CDT)
````

<a id="w003607"></a>

## W003607 — User:ChosenOreo/Chipsets

Bron-ID: W003607
Namespace: User (2)
Revisie: 16491 | Datum: 2014-06-21T06:47:42Z
Bron: https://wiki.osdev.org/User:ChosenOreo/Chipsets
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChosenOreo%2FChipsets&oldid=16491

````text
== Intel Chipsets ==
=== X58/ICH10 Family (Nehalem Microarchitecture) ===
[[User:ChosenOreo/Chipsets/Intel X58|X58 Northbridge]]<br />
[[User:ChosenOreo/Chipsets/Intel ICH10|ICH10 Southbridge]]
````

<a id="w003608"></a>

## W003608 — User talk:Chosenoreo

Bron-ID: W003608
Namespace: User talk (3)
Revisie: 16493 | Datum: 2014-06-21T06:49:03Z
Bron: https://wiki.osdev.org/User_talk:Chosenoreo
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AChosenoreo&oldid=16493

````text
Hi, this is my talk page. If you have anything you wish to discuss with me, this would be the place to do it. Criticism on articles I have written would be greatly appreciated. [[User:Chosenoreo|- ChosenOreo (acollado@citlink.net)]] 01:49, 21 June 2014 (CDT)
````

<a id="w003609"></a>

## W003609 — UEFI App Bare Bones

Bron-ID: W003609
Namespace: Main (0)
Revisie: 30312 | Datum: 2026-03-17T18:58:30Z
Bron: https://wiki.osdev.org/UEFI_App_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=UEFI_App_Bare_Bones&oldid=30312

````text
{{BeginnersWarning}}
{{Rating|2}}

In this tutorial, developers will create a hard drive or ISO image containing a bare bones UEFI application for the x86-64 platform.

It is recommended to have read and fully understood the [[Bare Bones]] tutorial first. The [[UEFI]] page provides some background to the UEFI boot process and should also be consulted first.

This tutorial uses the header files and GUID definitions from the [[GNU-EFI]] project, but does '''not''' use the gnu-efi build system, but rather the MinGW-w64 or LLVM/Clang toolchain.

== Prerequisites ==

Developers will need a [[GCC Cross-Compiler]] or Clang targeting the '''x86_64-w64-mingw32''' target (for [[PE]] output), and the [https://sourceforge.net/projects/gnu-efi/ '''gnu-efi'''] package (for UEFI headers). Most Linux distros provide cross-compilers for this target, so it's usually not necessary to build it yourself. This example does not link against [[GNU-EFI]] or follow its build process; only the headers are used.

To build the EFI filesystem image, developers can use [[MTools]] or [https://github.com/jncronin/mkgpt '''mkgpt'''] to create a hard disk image. To build a CD image, '''xorriso''' (in [[mkisofs]] emulation mode) will be needed. To run under an emulator, it is best to use '''qemu-system-x86_64''' coupled with the [http://tianocore.sourceforge.net/wiki/OVMF '''x64 OVMF firmware'''].

Under an apt-based system (e.g. Debian/Ubuntu), developers can run: <syntaxhighlight lang="bash">sudo apt-get install qemu ovmf gnu-efi binutils-mingw-w64 gcc-mingw-w64 xorriso mtools</syntaxhighlight>

To install mkgpt you can run these commands:
<syntaxhighlight lang="bash">
git clone https://github.com/jncronin/mkgpt.git
cd mkgpt
automake --add-missing
autoreconf
./configure
make
sudo make install
</syntaxhighlight>

== Testing the emulator ==

Now is a good time to check the emulator is working successfully with the OVMF firmware.

<syntaxhighlight lang="bash">qemu-system-x86_64 -L OVMF_dir/ -pflash OVMF.fd</syntaxhighlight>should launch qemu and launch a UEFI shell prompt.

== Preparing the files ==

=== hello.c ===

Next, create a file with the following:
<syntaxhighlight lang="c">
#include <efi.h>
#include <efilib.h>

EFI_STATUS efi_main(EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    EFI_STATUS Status;
    EFI_INPUT_KEY Key;

    /* Store the system table for future use in other functions */
    ST = SystemTable;

    /* Say hi */
    Status = ST->ConOut->OutputString(ST->ConOut, L"Hello World\r\n"); // EFI Applications use Unicode and CRLF, a la Windows
    if (EFI_ERROR(Status))
        return Status;

    /* Now wait for a keystroke before continuing, otherwise your
       message will flash off the screen before you see it.

       First, we need to empty the console input buffer to flush
       out any keystrokes entered before this point */
    Status = ST->ConIn->Reset(ST->ConIn, FALSE);
    if (EFI_ERROR(Status))
        return Status;

    /* Now wait until a key becomes available.  This is a simple
       polling implementation.  You could try and use the WaitForKey
       event instead if you like */
    while ((Status = ST->ConIn->ReadKeyStroke(ST->ConIn, &Key)) == EFI_NOT_READY) ;

    return Status;
}
</syntaxhighlight>

=== gnu-efi/lib/data.c ===

Developers will also need bring in the data.c file from the gnu-efi distribution, as this contains many predefined GUIDs for the various UEFI services. To avoid bloat and unnecessary dependencies on the rest of gnu-efi, it will need to be edited to remove the references to 'LibStubStriCmp', 'LibStubMetaiMatch', and 'LibStubStrLwrUpr' (simply set all the members of the LibStubUnicodeInterface structure be NULL).

=== gnu-efi/inc/lib.h ===

data.c includes this file. It must be copied as-is to the source directory.

== Building ==

To build, use the cross-compiler:
<syntaxhighlight lang="bash">
# compile: (flags before -o become CFLAGS in the Makefile)
x86_64-w64-mingw32-gcc -ffreestanding -Ipath/to/gnu-efi/inc -Ipath/to/gnu-efi/inc/x86_64 -Ipath/to/gnu-efi/inc/protocol -c -o hello.o hello.c
x86_64-w64-mingw32-gcc -ffreestanding -Ipath/to/gnu-efi/inc -Ipath/to/gnu-efi/inc/x86_64 -Ipath/to/gnu-efi/inc/protocol -c -o data.o path/to/gnu-efi/lib/data.c
# link: (flags before -o become LDFLAGS in the Makefile)
x86_64-w64-mingw32-gcc -nostdlib -Wl,-dll -shared -Wl,--subsystem,10 -e efi_main -o BOOTX64.EFI hello.o data.o
</syntaxhighlight>
Note here that '--subsystem 10' specifies an EFI application for ld.

=== Under LLVM/clang ===
The build sequence under LLVM/clang is essentially the same, although there is the advantage of having ''all'' targets installed by default:

<syntaxhighlight lang="bash">
CFLAGS='-target x86_64-unknown-windows 
        -ffreestanding 
        -fshort-wchar 
        -mno-red-zone 
        -Ipath/to/gnu-efi/inc -Ipath/to/gnu-efi/inc/x86_64 -Ipath/to/gnu-efi/inc/protocol'
LDFLAGS='-target x86_64-unknown-windows 
        -nostdlib 
        -Wl,-entry:efi_main 
        -Wl,-subsystem:efi_application 
        -fuse-ld=lld-link'
clang $CFLAGS -c -o hello.o hello.c
clang $CFLAGS -c -o data.o path/to/gnu-efi/lib/data.c
clang $LDFLAGS -o BOOTX64.EFI hello.o data.o
</syntaxhighlight>

Passing '--target x86_64-unknown-windows' to clang tells it to compile for x86_64 "Windows". This is quite not the same as 64-bit UEFI PE yet, but as before the "freestanding" part makes it a good kernel image. An example of this toolchain is found in the [https://github.com/c-util/c-efi c-efi] project.

Note the '-mno-red-zone' part used here as well -- it is a [https://stackoverflow.com/questions/25787408/why-cant-kernel-code-use-a-red-zone bad idea] to use a red zone for kernel code if interrupts are to be implemented. It should be done with GCC as well, but read [[Libgcc without red zone]] for the extra work needed to be done.

== Creating the FAT image ==
{{Main|Bootable Disk}}

Next, create a FAT filesystem image.
<syntaxhighlight lang="bash">
dd if=/dev/zero of=fat.img bs=1k count=1440
mformat -i fat.img -f 1440 ::
mmd -i fat.img ::/EFI
mmd -i fat.img ::/EFI/BOOT
mcopy -i fat.img BOOTX64.EFI ::/EFI/BOOT
</syntaxhighlight>

=== Running as a USB stick image ===

The FAT image can either be written directly to a USB stick and used in in a UEFI machine, or it can be run directly in QEMU:

<syntaxhighlight lang="bash">qemu-system-x86_64 -L OVMF_dir/ -pflash OVMF.fd -usb -usbdevice disk::fat.img</syntaxhighlight>

=== Creating and running the HD image ===

The HD image is a disk image in the [[GPT]] format, with the FAT image specially identified as a 'EFI System Partition'.

<syntaxhighlight lang="bash">
mkgpt -o hdimage.bin --image-size 4096 --part fat.img --type system
qemu-system-x86_64 -L OVMF_dir/ -pflash OVMF.fd -hda hdimage.bin
</syntaxhighlight>

=== Creating and running the CD image ===
The ISO image is a standard ISO9660 image which contains the FAT image as a file. A special El Torito option (-e) then points EFI aware systems to this image to be loaded. The CD image can either be burned to a CD and ran in a UEFI machine, or run directly in QEMU:
<syntaxhighlight lang="bash">
mkdir iso
cp fat.img iso
xorriso -as mkisofs -R -f -e fat.img -no-emul-boot -o cdimage.iso iso
qemu-system-x86_64 -L OVMF_dir/ -pflash OVMF.fd -cdrom cdimage.iso
</syntaxhighlight>

== What to do next? ==

Developers may want to try using some more of the EFI boot services, e.g., to [[Loading files under UEFI|read more files]] from the FAT image, manage memory, set up [[GOP|graphical frame buffer]] etc. (see the [http://www.uefi.org/specifications UEFI Specifications] page for further documentation of this).

There is also a finished app bare bone which supports both Linux and Windows (Visual Studio), see [https://github.com/pbatard/uefi-simple uefi-simple].

== Common problems ==

Some UEFI hardware implementations require that the FAT image is in the FAT32 format (rather than FAT12 or FAT16). OVMF does not have this limitation, so developers will not see such a problem in QEMU. However, the minimum size of a FAT32 filesystem is around 32 MiB, so developers will need to generate a much larger image and pass the '-F' option to mformat.

== See also ==

* [[UEFI]]
* [[UEFI ISO Bare Bones]]
* [[GNU-EFI]]
* [[POSIX-UEFI]]

[[Category:Bare bones tutorials]]
[[Category:Firmware]]
[[Category:UEFI]]
````

<a id="w003610"></a>

## W003610 — User:Brynet-inc

Bron-ID: W003610
Namespace: User (2)
Revisie: 24781 | Datum: 2020-06-17T12:50:40Z
Bron: https://wiki.osdev.org/User:Brynet-inc
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABrynet-inc&oldid=24781

````text
Hi! I'm Bryan.

== About Bryan ==
* Canadian
* OpenBSD slacker
* Likes pizza

== Links to Bryan ==
* [http://brynet.ca/ Website]
* [https://twitter.com/canadianbryan Twitter]
````

<a id="w003612"></a>

## W003612 — MessiahAndrw/Designing A Language

Bron-ID: W003612
Namespace: Main (0)
Revisie: 30160 | Datum: 2026-03-09T08:45:48Z
Bron: https://wiki.osdev.org/MessiahAndrw/Designing_A_Language
Revisie-URL: https://wiki.osdev.org/index.php?title=MessiahAndrw%2FDesigning_A_Language&oldid=30160

````text
This page is really a bunch of thoughts on designing programming languages.

Why would you want to design a language? Because it's fun. Perhaps you want to write your OS in your own language, or perhaps you just want to make your own scripting language.

In this section I'm not going to talk about the actual language itself and not get into [[User:No92/Compiler_Development|compiler development]].

Hopefully, by being interested in designing your own programming language, you have an idea of what kind of language you want to design. If not, figure it out now. Take the best of what you love from languages, and take out the bits you don't like, and make it your own.

== Pick a Paradigm ==
When you start designing your language, you need to come up with what sort of [http://en.wikipedia.org/wiki/Programming_paradigm programming paradigms[/url] you want to build your language around.

Do you want an event driven language? Do you want it a functional language where no function has side effects? Do you want a flow-based language where you define components and how data flows between them? Do you want it to be object oriented? Do you want it to be contract driven, or something completely different like time driven?

It is true that if a language is Turing complete you can support virtually any paradigm, even if not easily. But, you can make working with particular paradigms much easier by having first-class support for them in your programming language, or even design your language around them. For example - languages built around the class-based object-oriented paradigm tend to support classes as a high-level feature.

A large factor that determines what paradigms you will follow will largely depend on the intended usage of your language. Do you want it to be a general purpose language, that someone can write end-user applications in (like C, C#, Java)? Or do you want it domain specific?

Domain-specific languages are languages built for a specific purpose. Here are some examples of domain-specific languages:
* [http://en.wikipedia.org/wiki/Batch_file DOS batch files] use job control language which basically allows you to call a batch of programs, with limited support to piping one program's output to another program's input, and limited flow control.
* [http://en.wikipedia.org/wiki/GLSL GLSL] is a shading language for controlling the GPU. GLSL programs are short and sweet, with a focus on throughput and manipulating pixels, vertices, and other graphical elements in parallel.
* [http://en.wikipedia.org/wiki/Wolfram_(programming_language) The Wolfram Language] is a language built for math, simulation, and visualization.
* [http://en.wikipedia.org/wiki/Sql SQL] is a relational database language.

There are languages that look very difficult to how one usually imagines a programming language, for example:
* [http://en.wikipedia.org/wiki/XML XML] is a markup language for representing structured documents.
* [http://en.wikipedia.org/wiki/LaTeX LaTeX] is another markup language for typesetting and document preparation.

The rest of this article will focus on general purpose programming languages.

== Type System ==
In general purpose languages, you typically have variables which store values, references to objects, among other things. How you assign types to variables is referred to as the [http://en.wikipedia.org/wiki/Type_system type system]. There are two major kinds of type systems - static typing and dynamic typing.

Static typing refers to a type system where types are assigned at compile-time, and the programmer can easily determine what types are assigned to what variables. For example, if a variable has the type 'int' or 'string', you have a pretty good idea of what it is and how it is stored in memory, and that variable has a consistent type throughout its life.

Dynamic typing refers to a type system where types are not explicitly defined, and in many cases, the same variable may have different types at different times.


== Memory Safety ==

== Dynamic Evaluation ==

== Grammar ==
Learn 

 Backus–Naur Form
[[Category:Compiler Development]]
````

<a id="w003613"></a>

## W003613 — User:Lweb20

Bron-ID: W003613
Namespace: User (2)
Revisie: 18435 | Datum: 2015-11-01T23:24:55Z
Bron: https://wiki.osdev.org/User:Lweb20
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALweb20&oldid=18435

````text
== Optimized Functions ==

<source lang="C">
#define inline __attribute__((always_inline)) inline

inline void* memset(void* ptr, int value, size_t num)
{
	asm("rep stosb": : "a"(value), "c"(num), "D"(ptr));
	return ptr;
}

inline void* memsetw(void* ptr, int value, size_t num)
{
	asm("rep stosw": : "a"(value), "c"(num), "D"(ptr));
	return ptr;
}

inline void* memsetd(void* ptr, int value, size_t num)
{
	asm("rep stosd": : "a"(value), "c"(num), "D"(ptr));
	return ptr;
}

inline void* memcpy(void* destination, const void* source, size_t num)
{
	asm("rep movsb": : "D"(destination), "c"(num), "S"(source));
	return destination;
}
</source>

== VGA ==

=== VGA controllers ===

The VGA has controllers which in turn have registers to manipulate.

<source lang="C">
// attribute controller
#define VGA_ATTRIBUTE_CONTROLLER	0x3C0

// sequencer
#define VGA_SEQUENCER			0x3C4

// graphics controller
#define VGA_GRAPHICS_CONTROLLER		0x3CE

// crt controller
#define VGA_CRT_CONTROLLER		0x3D4
</source>

Some VGA ports are not associated with a specific controller. They contain a single register.

<source lang="C">
#define VGA_FEATURE_CONTROL			0x3DA

#define VGA_MISC_REG_WRITE			0x3C2
#define VGA_MISC_REG_READ			0x3CC
</source>

=== Useful functions ===

==== Write the registers ====

<source lang="C">
inline void vga_attrib_write(unsigned char index, unsigned char value)
{
	inb(VGA_FEATURE_CONTROL);
	outb(VGA_ATTRIBUTE_CONTROLLER, index);
	outb(VGA_ATTRIBUTE_CONTROLLER, value);
}

inline void vga_seq_write(unsigned char index, unsigned char value)
{
	outw(VGA_SEQUENCER, ((unsigned short)value << 8) | index);
}

inline void vga_graphics_write(unsigned char index, unsigned char value)
{
	outw(VGA_GRAPHICS_CONTROLLER, ((unsigned short)value << 8) | index);
}

inline void vga_crtc_write(unsigned char index, unsigned char value)
{
	outw(VGA_CRT_CONTROLLER, ((unsigned short)value << 8) | index);
}


inline void vga_misc_write(unsigned char value)
{
	outb(VGA_MISC_REG_WRITE, value);
}

inline void vga_feature_write(unsigned char value)
{
	outb(VGA_FEATURE_CONTROL, value);
}
</source>

To write registers 0 through 7 of the CRT is needed to unlock.

<source lang="C">
inline void vga_crtc_unlock()
{
	unsigned short data;
	
	outb(VGA_CRT_CONTROLLER, 0x11);
	data = inb(VGA_CRT_CONTROLLER + 1);

	outw(VGA_CRT_CONTROLLER, (ClearBit(data, 7) << 8) | 0x11);
}

inline void vga_crtc_lock()
{
	unsigned short data;
	
	outb(VGA_CRT_CONTROLLER, 0x11);
	data = inb(VGA_CRT_CONTROLLER + 1);

	outw(VGA_CRT_CONTROLLER, (SetBit(data, 7) << 8) | 0x11);
}
</source>

Reactivate the screen refresh disabled by writing to the controller attributes.

<source lang="C">
inline void vga_attrib_active_palette()
{
	outb(VGA_ATTRIBUTE_CONTROLLER, (1 << 5));
}
</source>

==== Read the registers ====

<source lang="C">
inline unsigned char vga_attrib_read(unsigned char index)
{
	unsigned char data;
	
	inb(VGA_FEATURE_CONTROL);
	outb(VGA_ATTRIBUTE_CONTROLLER, index);
	data = inb(VGA_ATTRIBUTE_CONTROLLER + 1);
	
	return data;
}

inline unsigned char vga_seq_read(unsigned char index)
{
	outb(VGA_SEQUENCER, index);
	return inb(VGA_SEQUENCER + 1);
}

inline unsigned char vga_graphics_read(unsigned char index)
{
	outb(VGA_GRAPHICS_CONTROLLER, index);
	return inb(VGA_GRAPHICS_CONTROLLER + 1);
}

inline unsigned char vga_crtc_read(unsigned char index)
{
	outb(VGA_CRT_CONTROLLER, index);
	return inb(VGA_CRT_CONTROLLER + 1);
}

inline unsigned char vga_misc_read()
{
	return inb(VGA_MISC_REG_READ);
}

inline unsigned char vga_feature_read()
{
	return inb(VGA_FEATURE_CONTROL);
}
</source>

==== Select plane ====

<source lang="C">
inline void vga_select_plane(unsigned char plane)
{
	vga_seq_write(0x02, plane);
}
</source>

== Wiki Links ==

'''Wiki:'''

http://wiki.osdev.org/OS_Specific_Toolchain

http://wiki.osdev.org/Hosted_GCC_Cross-Compiler

http://wiki.osdev.org/Porting_Newlib

'''Interrupt Controllers'''

8259 Programmable Interrupt Controller (PIC)

82489DX Advanced Programmable Interrupt Controller (APIC)

82093AA I/O Advanced Programmable Interrupt Controller (IOAPIC)


Cross Compiling:

http://f.osdev.org/viewtopic.php?f=13&t=18894

IOAPIC:

http://forum.osdev.org/viewtopic.php?t=23774&p=193091

http://forum.osdev.org/viewtopic.php?f=1&t=21745&start=0

SMP:

http://forum.osdev.org/viewtopic.php?f=1&t=23018 (Problems activating SMP on newer (core i5/i7) machines)

http://forum.osdev.org/viewtopic.php?t=28674 (IPI Question)

http://forum.osdev.org/viewtopic.php?f=1&t=23445 (CPUID - how to determine number of physical cores?)

== External Links ==

GZIP file format specification version 4.3

https://tools.ietf.org/html/rfc1952

Qemu Documentation

http://wiki.qemu.org/download/qemu-doc.html

Qemu Monitor

https://en.wikibooks.org/wiki/QEMU/Monitor

Bochs User Manual

http://bochs.sourceforge.net/doc/docbook/user/bochsrc.html

X.org Docs

http://www.x.org/docs/

The little book about OS development

https://littleosbook.github.io/

Linux System Calls

https://sourceware.org/newlib/libc.html#Syscalls

http://man7.org/linux/man-pages/man2/syscalls.2.html

http://syscalls.kernelgrok.com/

SMP

http://www.cheesecake.org/sac/smp.html

X86 architecture

https://en.wikipedia.org/wiki/X86

List of Intel CPU microarchitectures

https://en.wikipedia.org/wiki/List_of_Intel_CPU_microarchitectures

List of AMD CPU microarchitectures

https://en.wikipedia.org/wiki/List_of_AMD_CPU_microarchitectures
````

<a id="w003614"></a>

## W003614 — User talk:Lweb20

Bron-ID: W003614
Namespace: User talk (3)
Revisie: 16591 | Datum: 2014-08-04T18:20:32Z
Bron: https://wiki.osdev.org/User_talk:Lweb20
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ALweb20&oldid=16591

````text
== South-European language ==

The wiki is for predominantly English readers. It would help if you could make contributions in the appropriate language, so I moved it here so you have a chance to translate the content. - [[User:Combuster|Combuster]] 13:20, 4 August 2014 (CDT)
````

<a id="w003615"></a>

## W003615 — Libgcc without red zone

Bron-ID: W003615
Namespace: Main (0)
Revisie: 28491 | Datum: 2023-12-05T12:37:20Z
Bron: https://wiki.osdev.org/Libgcc_without_red_zone
Revisie-URL: https://wiki.osdev.org/index.php?title=Libgcc_without_red_zone&oldid=28491

````text
{{Main|Libgcc}}

This article discuses how to build [[libgcc]] without the need to support a <tt>red-zone</tt> in your kernel.
If you're not targeting [[X86-64|x86-64]] you don't need this as i*86 has no such requirements.

== What is the 'red-zone'? ==
The <tt>red-zone</tt> is a feature described in the [[System V ABI#x86-64|x86-64 ABI]].

It is a 128 byte long region located directly below the stack pointer. This region is free-for-use for the compiler without the requirement to notify the application / the os or any running interrupt handler.

For user applications there is no issue as interrupts and other kernel related code won't interfere with the user stack.
In your kernel however things can get ugly, especially so if you have nested interrupts and no <tt>red-zone</tt> support. 
Imagine running inside your interrupt handler, gcc puts some data inside the red zone, a nested interrupt occurs and clobbers the <tt>red-zone</tt> or vice versa.

To get around this the <tt>red-zone</tt> can be disabled by passing <tt>-mno-red-zone</tt> to [[GCC]].
<source lang="bash">x86_64-elf-gcc $CFLAGS -mno-red-zone ...</source>

== Why modify libgcc? ==
If you link against [[libgcc]] (as you should) there is one problem: [[libgcc]] is build with <tt>red-zone</tt> <u>enabled</u>.

So while your kernel works just fine the methods in [[libgcc]] may mess things up by accident.
The solution is simple - rebuild [[libgcc]] with <tt>-mno-red-zone</tt>. Fortunately [[GCC]] supports this in a straight forward way by providing [[multilib]] support inside it's source tree.

== Preparations ==
{{Main | GCC Cross-Compiler}}

Extract and prepare the [[GCC]] sources as described when building your [[GCC Cross-Compiler]] but <u>don't run configure just yet</u>.

Create the following file and save it as <tt>t-x86_64-elf</tt> inside <i><tt>gcc/config/i386/</tt></i> under your [[GCC]] sources.
<source lang="Make">
# Add libgcc multilib variant without red-zone requirement

MULTILIB_OPTIONS += mno-red-zone
MULTILIB_DIRNAMES += no-red-zone
</source>

By default this new configuration will not be used by [[GCC]] unless it's explicitly told to.
Open <tt>gcc/config.gcc</tt> in your favorite editor and search for case block like this:
<source lang="bash">
 x86_64-*-elf*)
 	tm_file="${tm_file} i386/unix.h i386/att.h elfos.h newlib-stdint.h i386/i386elf.h i386/x86-64.h"
 	;;
</source>

This is the target configuration used when creating a [[GCC Cross-Compiler]] for x86_64-elf.
Modify it to include the new [[multilib]] configuration:
<source lang="bash">
 x86_64-*-elf*)
	tmake_file="${tmake_file} i386/t-x86_64-elf" # include the new multilib configuration
	tm_file="${tm_file} i386/unix.h i386/att.h elfos.h newlib-stdint.h i386/i386elf.h i386/x86-64.h"
	;;
</source>

== Building libgcc ==

Run configure and then invoke the <tt>all-target-libgcc</tt> and <tt>install-target-libgcc</tt> as usual and [[GCC]] will build [[libgcc]] in two versions - one with <tt>red-zone</tt> enabled and one without.
You can check the successful build by checking the installed <tt>libgcc.a</tt> archives:
<source lang="bash">
find $TOOLCHAIN_PREFIX/lib -name 'libgcc.a'
</source>

If all went well you should see an additional [[libgcc]] installed in the <tt>no-red-zone</tt> multilib directory:
<source lang="text">
./gcc/x86_64-pc-elf/4.9.1/libgcc.a
./gcc/x86_64-pc-elf/4.9.1/no-red-zone/libgcc.a
</source>

== Linking against the correct multilib version ==
Assuming you're using [[GCC]] to link your kernel you're probably fine. All that's needed is to make sure <tt>-mno-red-zone</tt> is in your <tt>LDFLAGS</tt> when doing the final linker call.
<source lang="bash">
x86_64-elf-gcc $LDFLAGS -mno-red-zone -o kernel $SOURCES
</source>

If you're unsure which [[libgcc]] version is going to be used you can check by passing <tt>-mno-red-zone</tt> and <tt>-print-libgcc-file-name</tt> to [[GCC]]:
<source lang="bash">
x86_64-elf-gcc -mno-red-zone -print-libgcc-file-name
lib/gcc/x86_64-pc-elf/4.9.1/no-red-zone/libgcc.a
</source>

==See Also==
===Articles===
* [[GCC]]
* [[GCC Cross-Compiler]]
* [[Libgcc]]
* [[Bare Bones]]

[[Category:C]]
[[Category:C++]]
[[Category:Compilers]]
[[Category:FAQ]]
````

<a id="w003616"></a>

## W003616 — User:Xyh

Bron-ID: W003616
Namespace: User (2)
Revisie: 16724 | Datum: 2014-09-16T21:23:37Z
Bron: https://wiki.osdev.org/User:Xyh
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AXyh&oldid=16724

````text
My name is '''谢宇恒''' ('''XIE Yuheng''') (aka '''xyh''').
I am a math student (at the edge of dropping out) of an unknow college in China.

I am the author of [https://github.com/cicada-language/ cicada-language].
It is a very simple programming language.
It is a stack-based and functional language.
The design of cicada is influenced by [http://www.ccs.neu.edu/home/matthias/BTLS/ Scheme] and [http://www.forth.com/starting-forth/sf1/sf1.html Forth] and [http://www.kevinalbrecht.com/code/joy-mirror/joy.html Joy].

you can find my OS is [https://github.com/cicada-language/cicada here].

My email address is xyheme@gmail.com

BTW, I use fasm & archlinux & xmonad & emacs & org-mode,
and I am a fans of [http://www.art.net/~hopkins/Don/unix-haters/handbook.html The Unix-Haters Handbook].
````

<a id="w003617"></a>

## W003617 — Creating A Shell

Bron-ID: W003617
Namespace: Main (0)
Revisie: 19431 | Datum: 2016-07-12T08:20:00Z
Bron: https://wiki.osdev.org/Creating_A_Shell
Revisie-URL: https://wiki.osdev.org/index.php?title=Creating_A_Shell&oldid=19431

````text
{{In_Progress}}

Please note that the contents of this article may not be applicable to all operating system designs, and is modeled predominantly after the shells found on Windows and UNIX systems. All code in this article is based around a typical userspace API, using common or understandable names for the API functions.

=What is a shell?=
A shell is a userspace application that provides a simple interface to an operating system kernel. The simplest shell performs the following tasks:

# Execute other userspace binaries
# Provide input data for the binaries executed
# Provide a means for the executing binaries to output data

A shell may also offer other features depending on the operating system targeted, such as a means to set and use environment variables if such a concept exists in the kernel. The way that a shell is designed depends heavily on the operating system targeted, for example if the operating system has the concept of a "working directory" then the shell may want to maintain its own working directory which will become the working directory of any binaries executed (and provide commands to change the working directory, which may be a built-in command - a command handled by the shell itself, rather than an external binary - such as the <code>cd</code> command in the BASH shell).

Depending on the design of the kernel, the shell may provide input and output facilities through the "standard input" and "standard output" streams provided by the kernel to each executing binary. In this case, the shell will typically offer some form of output redirection, whereby input may be read from a file and output may be written to a file or passed as input to a second binary; unredirected input will typically be read from the shell's own "standard input" stream and unredirected output will typically be written to the shell's own "standard output" stream.

As a shell is a form of user interface, it will typically read input from the user (which may, as aforementioned, occur through its "standard input" stream) and display output to the user. It's important, however, to get the distinction between the physical hardware used to interact with the user and the shell's interface with the user. For example, the user may be working at a physical keyboard and monitor attached to the computer, or they may be connected over a network, or they may be using a serial console. A versatile shell should be able to work in all of these situations, reading input and sending output through whatever device the user is using (of course, this may not be possible in all operating systems, unless a concept analogous to the UNIX "standard input" and "standard output" is present in the kernel, or the kernel provides some other way of abstracting input devices or emulating one input device as another).

Typically, thus, a simple shell will do the following for each binary executed:

# Read a command from the user, which becomes the name of the binary to execute plus any command-line arguments (if applicable)
# Tell the kernel to execute the binary
# Read input from the user and send it to the binary
# Read output from the binary and send it to the user
# Repeat steps 3 and 4 until the binary terminates

In order to do this, the shell will require the following:

# A kernel with an API call for executing a binary
# A way to get input from the user (typically, this requires a keyboard driver for the kernel and an API call for reading from the keyboard - may take the form of "standard input")
# A way to send output to the user (typically, this requires a video driver for the kernel and an API call for printing text to the screen - may take the form of "standard output")

It's probably also desirable for the user to be able to edit their commands without having to type them all over again, so a set of string editing functions (ala GNU readline) are also preferable.

=Outline of shell=
Based on the explanations above, here is the main function for a simple shell:

<source lang="c">
void main(int argc, char* argv[]) // edit as appropriate for your kernel
{
    while (true) // you may want to provide a built-in "exit" command
    {
        char* command;
        int proc;
standard I/O streams
        output_prompt();               // display a prompt
        command = input_line();        // get a line of input, which will become the command to execute
        proc = process_start(command); // start a process from the command
        free(command);

        while (process_executing(proc))
        {
            if (input_line_waiting())
            {
                char* line;
                line = input_line();                 // read input from user
                process_send_input_line(proc, line); // send input to process
                free(line);
            }
            if (process_output_line_waiting(proc))
            {
                char* output;
                output = process_get_output_line(proc); // get output from process
                output_line(output);                    // write output to user
                free(output);
            }
        }
    }
}
</source>

=Implementation=
The implementation of the functions referred to in the above code snippet depends heavily on the design of your operating system, but here are some typical examples:

<source lang="c">
static void output_prompt()
{
    output_line(">");
}

static void output_line(char* line)
{
    printf("%s\n", line);
}

static int input_line_waiting()
{
    // TODO: perhaps someone more familiar with the C library could write a better example?
}

static char* input_line()
{
    // TODO: perhaps someone more familiar with the C library could write a better example?
    // TODO: this function will allocate a char buffer containing the line read
}

static int process_start(char* command)
{
    exec(command);
    // TODO: needs to return an int identifying this process
}

static int process_executing(int proc)
{
    // TODO: what would be a good example here?
}

static void process_send_input_line(int proc, char* line)
{
    // TODO: what would be a good example here?
}

static int process_output_line_waiting(int proc)
{
    // TODO: what would be a good example here?
}

static char* process_get_output_line(int proc)
{
    // TODO: what would be a good example here?
    // TODO: this function will allocate a char buffer containing the line read
}
</source>

=TODO=
* Where to include string editing functions?
* How to implement working directory, I/O redirection/piping, etc.?

=External Links=
* http://stephen-brennan.com/2015/01/16/write-a-shell-in-c/

<!-- Hello, this is a guide for writing a simple shell. Our shell will need a few things:

  1. A working bootloader/GRUB
  2. An IDT and IRQ handler
  3. A GDT
  4. A keyboard driver
  5. A working text input/output system.
  6. A set of string editing functions

First we need to make a file, shell.c, this file will contain all our functions and shell managers. The first function we need to write is our initialization function then our actual shell function. 

<source lang="c">
/* shell.c */
void init_shell()
{
}
  
void shell()
{
}
</source>

Now you need to add "init_shell()" to your setup routine in your kernel.c file or whatever you named it, followed by a while loop running shell until the boolean "exit_status" is true. 

Ok, now we need to write a few things. The first is command table, or a way to store our commands. I do this with a structure as follows:

<source lang="c">
/* shell.h */
#define MAX_COMMANDS 100

typedef struct
{
    char *name;
    char *description;
    void *function;
} command_table_t;
</source>

Now we have a way to store commands, but how do we use this? Well lets add a few variables to shell.c:

<source lang="c">
/* shell.c */
command_table_t CommandTable[MAX_COMMANDS];
</source>

This goes right above the function definitions. The first is a way to keep track of how many commands there are. The second is our command table. Ok, so now we have a "Command Table", but how to we access it? Well in order to add a command we need a function to do it. Add this below the other functions:

<source lang="c">
/* shell.c */
void add_new_command(char *name, char* description, void *function)
{
    if(NumberOfCommands + 1 < MAX_COMMANDS)
    {
        NumberOfCommands++;

        CommandTable[NumberOfCommands].name = name;
        CommandTable[NumberOfCommands].description = description;
        CommandTable[NumberOfCommands].function = function;
    }
    return;
}
</source>

Ok, now we are getting somewhere. We can add a command. But what about getting command line input? Thats up to our "shell()" function. But before we code that, lets add another variable: "char* input_string". Now for the shell function:

<source lang="c">
/* shell.c */
void shell()
{
    puts("\nMy_Prompt>");
    gets(input_string);

    void (*command_run)(void);
}
</source>

Ok, now we can get a string. But what about finding what command the user typed? Well we rely on old trusty "findCommand()" function for that one:

<source lang="c">
/* shell.c */
int findCommand(char* command)
{
    int i;
    int ret;

    for(i = 0; i < NumberOfCommands + 1; i++)
    {
        ret = strcmp(command, CommandTable[i].name);

        if(ret == 0)
            return i;
        else
            return -1;
    }
}
</source>

Now we need to add a little something right below "gets(input_string)", but above "void (*command_function)(void)":

<source lang="c">
/* shell.c */
int i = findCommand(input_string);

if(i >= 0)
{
    command_function = CommandTable[i].function
    command_function();
}
else
{
    return;
}
    
return;
</source>

Now in our "init_shell()" function we need to add a few commands:

<source lang="c">
/* shell.c */
add_new_command("help", "You code this one.", help_command);
add_new_command("", "", empty_command);
</source>

Now in our shell.h file:

<source lang="c">
extern void add_new_command();
extern void help_command();
extern void empty_command();
</source>

Now you need to code a void help_command function and a empty_command that manages null input, all it needs is a definition:

<source lang="c">
void empty_command()
{
}
</source>

An idea is to loop through all the command table and print the command name and description - for the help command. 

Whats left?

Add support for filesystem commands, a clear screen command, I don't know. But enjoy developing :)

- wxwsk8er

==External Links==
* http://stephen-brennan.com/2015/01/16/write-a-shell-in-c/ Write a Shell in C - similar concepts, but for user space. -->
````

<a id="w003618"></a>

## W003618 — File:Rand spectrum.png

Bron-ID: W003618
Namespace: File (6)
Revisie: 16613 | Datum: 2014-08-15T15:49:48Z
Bron: https://wiki.osdev.org/File:Rand_spectrum.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ARand_spectrum.png&oldid=16613

````text
Frequency spectrum of a standard LCG implementation of rand() versus an LFSR implementation.
````

<a id="w003619"></a>

## W003619 — Talk:Random Number Generator

Bron-ID: W003619
Namespace: Talk (1)
Revisie: 16614 | Datum: 2014-08-15T15:52:30Z
Bron: https://wiki.osdev.org/Talk:Random_Number_Generator
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARandom_Number_Generator&oldid=16614

````text

== Power Spectral Density of Linear feedback shift registers  ==

For those concerned about even power spectral density (aka white noise), it should be noted that LFSRs tend to have a bias towards lower frequencies (similar to pink noise).

Here's a comparison of the frequency spectrum of glibc's rand() vs an LFSR:

[[File:Rand_spectrum.png|400px|left]]
````

<a id="w003620"></a>

## W003620 — User:Muazzam

Bron-ID: W003620
Namespace: User (2)
Revisie: 18183 | Datum: 2015-06-02T09:56:24Z
Bron: https://wiki.osdev.org/User:Muazzam
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMuazzam&oldid=18183

````text
Hello, I am Muazzam Ali Kazmi, hobbyist programmer from Layyah, Pakistan. I am a 11th-grade student and started programming about 4 years ago. [http://alotware.hol.es Alotware] is my current OS project. My interests include Religion(s), Operating Systems, Virtual machines, Simulating a universe in Computers, Compilers, Theoretical Computer Science, Artificial Intelligence, Math and Science. 
----

Email: mailto:muazzam_ali@live.com
----

Facebook: http://www.facebook.com/muazzamalikazmi5
````

<a id="w003621"></a>

## W003621 — User talk:Muazzam

Bron-ID: W003621
Namespace: User talk (3)
Revisie: 17493 | Datum: 2015-01-26T10:07:17Z
Bron: https://wiki.osdev.org/User_talk:Muazzam
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMuazzam&oldid=17493

````text

````

<a id="w003622"></a>

## W003622 — User:Minoto

Bron-ID: W003622
Namespace: User (2)
Revisie: 16624 | Datum: 2014-08-24T02:34:03Z
Bron: https://wiki.osdev.org/User:Minoto
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMinoto&oldid=16624

````text
== My Background ==

I'm primarily a hobby programmer.  My first exposure to computers and programming came in the late 1970s / early 1980s, trying out small things in different Basic dialects on Apple IIs, TRS-80s, Atari 400s and 800s, and other microcomputers of the time.  My own first computer was a TI-99/4A, which was followed a few years later by an IBM PCjr and a succession of PC clones.  Two books that had a big influence on me back then were Steven Levy's ''Hackers'' and Tracy Kidder's ''The Soul of a New Machine''; together, they left me with a long-standing interest in getting my hands dirty and having fun with hardware, though that was a lot harder to follow up on back then than it is now.  Rodnay Zaks' ''Programming the Z80'' was another, as were Steve Ciarcia's ''Build Your Own Z80-Based Computer'' and R.G. Loeliger's ''Threaded Interpretive Languages'' when I stumbled across them later, along with Tanenbaum's ''Modern Operating Systems''.

Languages that I've used or found interesting, roughly in order, include Basic, Logo, Fortran, Pascal, Z80 assembly, C, Lisp, Java, C++, Forth, and Haskell, though I wouldn't call myself an expert at any of them.

I've probably spent the most of my time using Windows, though I've also used PC/MS-DOS and still use BSD and Linux.

== How I Got Here ==

I originally found this site when I was working on an old-fashioned, the-language-is-the-OS implementation of Forth, and looking for information about working with the PC keyboard controller and floppy or hard disk controller directly, so I could remove my reliance on the BIOS and leave real mode behind.  These days, I alternately find that idea pointless / still worth doing, so I continue to make slow progress on it, along with other things.

== What I'd Like to Create ==

A programming environment like what I had back when I first got hooked on computers and programming -- a machine that I have full control over, that's ready to go within seconds of being turned on, that's fully interactive and cooperates with me, instead of throwing obstacle after obstacle in my way.  Truly personal computers have been affordable for decades now, and get more powerful every year; why hobble them with limitations from the resource-starved timesharing environments of the 1960s?  If I want to quickly draw something on the screen, I want to just do it, not work with multiple libraries to eventually get indirect access to be able to do so.

No, this isn't something that would be appropriate for a Web or database server.

Yes, the layers and layers of frameworks that have grown over the years have made it possible to do some pretty amazing things by writing only a small amount of new code.

But, at the end of the day, I'd like to be able to do what I want on my own machine with a minimum of interference, while taking advantage of the capabilities of modern hardware.

== What I've Contributed ==

Just basic typo corrections so far.
````

<a id="w003623"></a>

## W003623 — BIOS32

Bron-ID: W003623
Namespace: Main (0)
Revisie: 30199 | Datum: 2026-03-11T21:16:30Z
Bron: https://wiki.osdev.org/BIOS32
Revisie-URL: https://wiki.osdev.org/index.php?title=BIOS32&oldid=30199

````text
{{In Progress}}

== Introduction ==
'''BIOS32''' is a method for calling [[BIOS]] services from either 16-Bit [[Real Mode]], 16-Bit [[Protected Mode]] or 32-Bit Protected Mode. This method can be used to find BIOS32 services and execute BIOS32 methods, but it does not allow you to call "legacy" BIOS calls, directly, like INT 0x10 and INT 0x13 methods.  It is mainly used for [[Plug-and-Play|PnP]] and [[PCI]] services, but there may be other services available, as well.  These services are not well documented, and are not necessarily supported on all systems.  The BIOS32 API provides a method to query the system for support for a specific service, but it does not provide any method for getting a complete list of supported services.

== Interface ==
=== Locating ===
The BIOS32 ''Service Directory'' Header is located on a paragraph (16 Byte) boundary in the physical memory region <tt>0xE0000</tt>-<tt>0xFFFFF.</tt>
It starts with a 4 byte signature of <tt>_32_</tt> (<tt>0x5F32335F</tt>)

=== Structure ===
{| class="wikitable"
|+ Service Directory Header
! Offset || Size || Name || Description
|-
|0 || 4 bytes || Signature || ASCII signature: <tt>_32_</tt> (<tt>0x5F32335F</tt> in a DWORD integer).
|-
|4 || 4 bytes || Entry Point || 32 bit physical address (not <tt>CS:IP</tt>) of the BIOS32 calling interface entry point.
|-
|8 || 1 byte || Revision || Revision of BIOS32. Currently this is <tt>0</tt>.
|-
|9 || 1 byte || Length || Length of structure in paragraphs (units of 16 bytes).
|-
|10 || 1 byte || Checksum || Checksum of this header (1 byte granularity).
|-
|11 || 5 bytes || Reserved || Reserved for future use, should be <tt>0</tt>.
|}

=== Checksum ===
When the ''Service Directory Header'' header is found, a checksum should be performed. This is done by adding all the bytes of the header and then checking the result value, which must be equal to <tt>0</tt>.

=== Service Availability ===
In order to verify that a service is available, you must FAR CALL the 32-bit address located at offset 4 in the structure above, with the EBX register set to zero, and the EAX register set to the Component ID for the service that you are looking for.  If the service is available, the AL register will contain a zero, EBX will contain the base address for the service, ECX will contain the length of the service in bytes, and EDX will contain the offset (from EBX) for the entry point for the service.

If the service is not supported, the AL register will contain either 0x80 or 0x81.  (The documentation is ambiguous on this specific value, so it may be better to check for a non-zero value)

== Well Known Services ==
The services listed below have been found on real hardware, but may not be supported by some (or even most) systems.

{| class="wikitable"
|+ Service List
! Component ID
! Name
|-
|"$PnP" || Plug-n-Play BIOS
|-
|"$PCI" || PCI BIOS
|-
|"$PIR" || PCI IRQ Routing Table
|-
|"$PMM" || POST Memory Manager
|-
|"$BC$" || PXE Base Code (BC) Rom ID
|-
|"$ACF" || Plug-n-Play Auto Config
|-
|"$BLK" || Unknown
|-
|"$WDS" || Unknown
|-
|"$SNY" || Unknown
|-
|"$SDS" || Unknown
|-
|"$SDM" || Unknown
|-
|"$IRT" || Unknown
|-
|"$CFD" || Unknown
|-
|"$BBS" || Unknown
|-
|"$ASF" || Unknown
|-
|"$$CT" || Unknown
|-
|"MPTN" || Unknown
|-
|"PCI " || Unknown
|-
|}

== See Also ==
=== Articles ===
* [[BIOS]]
* [[PCI]]

=== External Links ===
* [https://web.archive.org/web/20150810181814/http://www-pc.uni-regensburg.de/hardware/TECHDOK/BIOS_32BIT_SERVICE.PDF Phoenix BIOS 32-bit service proposal]
* [https://groups.google.com/g/alt.os.development/c/pgREOB7YEws/m/J3UZ1sbvQnsJ Google Groups post about known services]
* [https://www.superfrink.net/athenaeum/OS-FAQ/os-faq-pci.html PCI BIOS32 call examples]

[[Category:BIOS]]
[[Category:X86]]
````

<a id="w003624"></a>

## W003624 — Category:BIOS

Bron-ID: W003624
Namespace: Category (14)
Revisie: 20360 | Datum: 2017-02-11T17:28:17Z
Bron: https://wiki.osdev.org/Category:BIOS
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABIOS&oldid=20360

````text
The BIOS is the resident firmware on the x86 architecture. It is almost exclusively 16 bit real mode code. It provides basic services to the boot-loader and 16 bit operating systems like DOS.

[[Category:X86]]
[[Category:Firmware]]
````

<a id="w003625"></a>

## W003625 — User:Akasei

Bron-ID: W003625
Namespace: User (2)
Revisie: 26609 | Datum: 2021-10-27T17:39:20Z
Bron: https://wiki.osdev.org/User:Akasei
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAkasei&oldid=26609

````text
[http://blackdev.org/ blackdev.org]
````

<a id="w003627"></a>

## W003627 — User:Requimrar/FAT32

Bron-ID: W003627
Namespace: User (2)
Revisie: 16660 | Datum: 2014-08-30T05:30:40Z
Bron: https://wiki.osdev.org/User:Requimrar/FAT32
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ARequimrar%2FFAT32&oldid=16660

````text
== FAT32 Implementation ==

Hello. Unlike the mainline [[FAT]] article on the Wiki, this is entirely focused on FAT32 and that only. It also should have no naming inconsistencies.
Note that my codebase is in C++, and the FAT32 driver is an object, so "this->" is seen frequently. Do not expect to copy and paste code. Also, this article will not have tables or whatever reference material, such as byte offsets for each value, etc. The closest you'll get to that here is a struct {} definition. For the actual spec, use either the fatgen103 document or [http://en.wikipedia.org/wiki/Design_of_the_FAT_file_system the Wikipedia article].

The code below is entirely to explain how FAT32 works. It is not meant to be implemented in an actual system. With an actual driver you need more abstraction, and should be dealing with VFS nodes and not cluster numbers, for instance.

There are also fictitious types, such as List<> and Vector<> (the angled brackets are c++ generic templates), it is up to you, the reader, to implement this yourself. It also uses fictitious methods like AllocateBuffer, foreach(type name in list), etc. This will not compile, and don't expect it to. 

== Helper Functions ==
First up, some helper functions. Make these file-static or something.
<source lang="C">
struct DirectoryEntry
{
	char name[8];
	char ext[3];
	uint8_t attrib;
	uint8_t userattrib;

	char undelete;
	uint16_t createtime;
	uint16_t createdate;
	uint16_t accessdate;
	uint16_t clusterhigh;

	uint16_t modifiedtime;
	uint16_t modifieddate;
	uint16_t clusterlow;
	uint32_t filesize;

} __attribute__ ((packed));


uint64_t FAT32FSDriver::ClusterToLBA(uint32_t cluster)
{
	return this->FirstUsableCluster + cluster * this->SectorsPerCluster - (2 * this->SectorsPerCluster);
}
</source>

The first struct should be fairly self-explanatory, it is simply a packed struct representing the values in a directory entry. The second function is used to convert a cluster number (say cluster 4) into an LBA address you can use to read from a disk (like 4412). Do not worry about the values, I'll explain them later.





== BPB Setup ==

I'll just provide code here. It should be self explanatory. This is also where FirstUsableCluster is defined, it is simply the first data cluster in the FAT volume. (ie. after the two FATS)
<source lang="C">
this->BytesPerSector		= *((uint16_t*)((uintptr_t) fat + 11));

this->SectorsPerCluster		= *((uint8_t*)((uintptr_t) fat + 13));
this->ReservedSectors		= *((uint16_t*)((uintptr_t) fat + 14));
this->NumberOfFATs		= *((uint8_t*)((uintptr_t) fat + 16));
this->NumberOfDirectories	= *((uint16_t*)((uintptr_t) fat + 17));


if((uint16_t)(*((uint16_t*)(fat + 17))) > 0)
	this->TotalSectors	= *((uint16_t*)((uintptr_t) fat + 17));

else
	this->TotalSectors	= *((uint32_t*)((uintptr_t) fat + 32));

this->HiddenSectors		= *((uint32_t*)((uintptr_t) fat + 28));

this->FATSectorSize		= *((uint32_t*)((uintptr_t) fat + 36));
this->RootDirectoryCluster	= *((uint32_t*)((uintptr_t) fat + 44));
this->FSInfoCluster		= *((uint16_t*)((uintptr_t) fat + 48));

this->BackupBootCluster		= *((uint16_t*)((uintptr_t) fat + 50));
this->FirstUsableCluster	= this->partition->GetStartLBA() + this->ReservedSectors
					+ (this->NumberOfFATs * this->FATSectorSize);
</source>

As I said, it should be fairly self explanatory, since the variable names are quite verbose. One point to note: since we're dealing only with FAT32, we can assume a FAT32 EBPB exists. In your own code, check that you're actually dealing with a FAT32 volume.

== Traversing a Path ==

I shall assume that you know how to split a path like /usr/local/lib/libc.a into its named components, and I assume you know how to iterate through a list to match them up. The following code will not take care of nuances you need to be aware of, such as the same name on different levels, eg. you have /folder/name and /folder/something/name. There are probably more efficient ways to write this.

<source lang="C">
// The actual function will probably return a value, such as a found/not found, or a VFS node or whatever.
// Again, this is not meant to be copied.
void Traverse(const char* path)
{
	// or whatever here, basically split the paths into its components.
	List<const char*> pathparts = split(path, '/');

	// get the root directory.
	List<FSObject>* dirs = ReadDir(this->RootDirectoryCluster);

	// iterate through it.
	foreach(FSObject fso in dirs)
	{
		// again, FSObject should be a VFS node of some kind, with FS specific information stored in a pointer within.
		// I'll just assume it has a member called 'cluster' at this point.

		// TODO
	}
}
</source>



<source lang="C">
// The following function takes a cluster number of a directory, then lists its contents.
List<FSObject>* ReadDir(uint32_t cluster)
{
	// if the cluster number is zero, assume we're trying to read from the root directory.
	if(cluster == 0)
		cluster = this->RootDirectoryCluster;


	// Get the cluster chain of the directory. This contains the directory entries themselves.
	// We will implement GetClusterChain later.
	uint64_t numclus = 0;	
	List<uint32_t>* clusters = this->GetClusterChain(node, &numclus);


	// try and read each cluster into a contiguous buffer.
	
	// this is the total number of bytes the directory is (in terms of on-disk dirents)
	uint64_t dirsize = numclus * this->SectorsPerCluster * 512;

	// allocate a buffer, rounding up the size to a page.
	uint64_t buf = AllocateBuffer(numclus * this->SectorsPerCluster * 512);
	uint64_t obuf = buf;

	assert(clusters);
	foreach(uint32_t cluster in clusters)
	{
		// read "SectorsPerCluster * 512" bytes at the LBA "ClusterToLBA(cluster)" into buf.
		ReadFromDisk(ClusterToLBA(cluster), buf, this->SectorsPerCluster * 512);

		// increment buf, so we don't trash its contents.
		buf += this->SectorsPerCluster * 512;
	}
	buf = obuf;

	// we now have the complete data of the directory.
	// ... to be implemented
}

</source>

This is the basic framework for reading a directory's contents: get its cluster (which is provided as an argument), get the cluster chain from the FAT table, read all the clusters into a buffer. It should be trivial to, for instance, do a for or while loop to iterate through each DirectoryEntry, incrementing your pointer offset by <code>sizeof(DirectoryEntry)</code> each time.

You may have noticed the missing GetClusterChain() method, which returns a list of clusters (since FAT is a linked list filesystem), gotten from the FAT. We will look at it now.

== Reading a Cluster Chain ==

<source lang="C">
List<uint32_t>* GetClusterChain(uint32_t firstcluster, uint64_t* numclus)
{
	// setup some stuff.
	uint32_t Cluster = firstcluster;
	uint32_t cchain = 0;
	List<uint32_t>* ret = new List<uint32_t>();

	// here we assume your 'ReadFromDisk' only does 512 bytes at a time or something.
	auto buf = AllocateBuffer(512);
	auto obuf = buf;
	do
	{
		// these formulas are gotten from the 'fatgen103' document.
		// 'FatSector' is the LBA of the disk at which the FAT entry you want is located.
		// 'FatOffset' is the offset in bytes from the beginning of 'FatSector' to the cluster.
		uint32_t FatSector = (uint32_t) this->partition->GetStartLBA() + this->ReservedSectors + ((Cluster * 4) / 512);
		uint32_t FatOffset = (Cluster * 4) % 512;

		// read 512 bytes into buf.
		ReadFromDisk(FatSector, buf, 512);

		// cast to an array so we get an easier time.
		uint8_t* clusterchain = (uint8_t*) buf;

		// using FatOffset, we just index into the array to get the value we want.
		cchain = *((uint32_t*)&clusterchain[FatOffset]) & 0x0FFFFFFF;

		// the value of 'Cluster' will change by the next iteration.
		// Because we're nice people, we need to include the first cluster in the list of clusters we return.
		ret->push_back(Cluster);

		// since cchain is the next cluster in the list, we just modify the things, shouldn't be too hard to grasp.
		Cluster = cchain;

		// numclus tells the caller how many clusters are in the chain.
		(*numclus)++;

	} while((cchain != 0) && !((cchain & 0x0FFFFFFF) >= 0x0FFFFFF8));	

	// check that there's more entries in the chain, if not break the loop.
	// the reason for using a do-while should be obvious.

	return ret;
}
</source>

Most of the difficulty in understanding FAT is the math, calculating the sectors and clusters and whatnot. The above function is as simple as it gets. It reads one sector of the FAT at a time, then follows the linked list. You can add optimisations, such as reading more than 512 bytes at a time, or checking if you even need to read again (since many FAT entries are consecutive, especially on a nearly-empty volume)

== Allocating a Cluster ==
This begins write support. I have yet to port that functionality from my old kernel into my current one, so this will take a while.

TODO.



== Conclusion ==
I hope that this article gave you a clearer and less confusing picture on how FAT32 works and the concepts behind it. Feel free to find me on irc at #osdev: @zhiayang, or anybody else online.
````

<a id="w003628"></a>

## W003628 — Cross-Porting Software

Bron-ID: W003628
Namespace: Main (0)
Revisie: 28165 | Datum: 2023-07-10T14:49:59Z
Bron: https://wiki.osdev.org/Cross-Porting_Software
Revisie-URL: https://wiki.osdev.org/index.php?title=Cross-Porting_Software&oldid=28165

````text
{{Rating|3}}

This page is notes on how to port software to your operating system by cross-compiling it. Many core packages use autoconf-generated ./configure script that provide a convenient interface for our purposes. This assumes your operating is somewhat Unix-like. There is a lot of subtle semantics that first-time porters often get wrong and particular packages occasionally misbehave. This tutorial assumes you are porting a package using a conventional configure script (generated with autoconf), but the principles can be adapted to other build systems.

== Prerequisites ==

This is an advanced matter and your operating system needs to have an established user-space and a dedicated toolchain:

* You need a [[OS Specific Toolchain]] like <tt>x86_64-myos</tt>.
* You need a [[C Library|standard library]] implementation that provides what the third party software needs.

== System Root ==
:Please read this [[Meaty_Skeleton#System_Root|article's section on sysroots]].

Your build process needs to involve a system root, a fake root directory for your operating system. Your cross-compiler must be set up to search this directory tree for libraries and headers. I recommend for your build system to look through each of your subprojects and ''make install'' them into this system root, then do the same for ports, and finally your build system extracts the desired files from the system root and makes a boot-able image. We will be installing ports into this directory tree. Futher ports may depend on previous ones and it's crucial the system root is searched.

== Prefixes ==

The build systems of the software installs the software somewhere. This is normally in <tt>${PREFIX}/bin</tt>, <tt>${PREFIX}/lib</tt> and so on. The packages often defaults to <tt>/usr/local</tt>, but this is generally used for site-specific files rather than vendor-provided files (you are a vendor now). Note how if you want to install packages into <tt>/bin</tt> and <tt>/lib</tt>, you set the prefix to the empty string, not a single slash. A prefix is not a directory path in its own right, but something that is added in front of a real directory path.

== Dependencies ==

Before porting a package, you need to port its dependencies first (and transitively their dependencies as well). You can often find a list of dependencies in the documentation (perhaps look for README or INSTALL files). Try also to run <tt>./configure --help</tt>, this will often list <tt>--with-foo</tt> options if the package depends on libfoo, it will also list which dependencies are optional. See also guides like [http://www.linuxfromscratch.org/blfs/ Beyond Linux from Scratch] as it contains useful dependency information.

You should be able to construct a nice directed acylic graph of packages and their dependencies (some edges being optional) and use it to decide the order packages are built. If you are not interested in optional dependencies, you can skip them, but beware: Later package may hard-depend on the package you skipped, and may assume a library was built with support for the packager you skipped.

== Patches ==

It is very likely that you will need to patch the packages you port. You need to set yourself up such that you can easily patch the packages with no to low cost in effort. You can help other people by hosting your patch collection somewhere public. It is often nicer to actually fix misbehaving packages than working around the problem in your build system in complex manners.

== Source Code ==

You need to find the source code for the package you wish to port. This is rather obvious. Generally, it's best to find the latest stable tarball of the package use that. This is preferable to using a git checkout of the package. It's a good idea to save a copy of the original tarball.

It's advisable to check whether this is a real release or whether it has been maliciously modified. Many projects provide hash values of their releases or otherwise sign their releases. Configure scripts are highly convenient places to hide malware (if you have the autoconf skills, you can regenerate the configure script and other files and see if they match). Man-in-the-middle attacking in insecure http/ftp download of a tarball is trivial, you can mitigate this danger by downloading from multiple networks and verifying what other people have.

You also definitely wish to to read the license for the software and whether it even permits the efforts we are taking here.

You extract the source code somewhere appropriate:

<source lang="bash">
# Use --extract --file if you have a hard time surviving xkcd 1168.
tar -xf libfoo-4.2.tar.xz
</source>

== pkg-config ==

Libraries increasingly provide pkg-config files that describe where the headers are installed and how to link against the library (and private library dependencies if statically linked). Working with pkg-config is preferable to fighting it (see below on packages rolling their own foo-config program) and it nicely supports system roots and is cross-compile aware. It's possible to compile a custom cross-pkg-config, or you can simply wrap your system one. Make a <tt>x86_64-myos-pkg-config</tt> executable shell script and put it somewhere in your path for the duration of the cross-compilation:

<source lang="bash">
#!/bin/sh
# Fill these in appropriately:
export PKG_CONFIG_SYSROOT_DIR=$MYOS_SYSROOT
export PKG_CONFIG_LIBDIR=$MYOS_SYSROOT/usr/lib/pkgconfig
# TODO: If it works this should probably just be set to the empty string.
export PKG_CONFIG_PATH=$PKG_CONFIG_LIBDIR
# Use --static here if your OS only has static linking.
# TODO: Perhaps it's a bug in the libraries if their pkg-config files doesn't
#       record that only static libraries were built.
exec pkg-config --static "$@"
</source>

You then set <tt>PKG_CONFIG=x86_64-myos-pkg-config</tt> to packages use your custom pkg-config instead, and <tt>PKG_CONFIG_FOR_BUILD=pkg-config</tt> so packages that wish to compile local programs use the system pkg-config.

== Build ==

This section lists the steps that are sufficient for an ideal port, but see below.

First you wish to find the <tt>config.sub</tt> file. The GNU coding conventions usually place it at <tt>build-aux/config.sub</tt> to avoid clutter in the main directory. You wish to add your operating system's target name to it, as you did in the [[OS Specific Toolchain]] article. This change is required, or otherwise the <tt>--host</tt> value will be rejected. This simple fact means that you likely have to have a small patch for every single port.

Secondly, you cross-build the package in this manner:

<source lang="bash">
# Potentially unset CC and such here (see configure --help) to prevent local
# tools from being mistakenly used as cross-tools. Alternatively, set them to
# your cross-tools unconditionally. Note also the existence of CC_FOR_BUILD in
# the case of packages that needs to build local tools to build themselves.

./configure --host=x86_64-myos --prefix=/usr
make
make DESTDIR=$SYSROOT install
</source>

This will cross-compile the software and install it under <tt>/usr</tt> on your system, inside your system root as the temporary installation location. Note how many packages remember <tt>--prefix</tt> and use it at runtime to locate their data files. You must not do <tt>--prefix=$SYSROOT/usr</tt> as that means libfoo would look files in <tt>/home/myuser/myos/sysroot/usr/share/libfoo</tt> while running on your operating system instead of <tt>/usr/share/libfoo</tt>. The <tt>DESTDIR</tt> acts as a second prefix for the purpose of installation, it's not revealed to the package before the install step, so it won't mistakenly remember it.

Alternatively, instead of setting <tt>DESTDIR</tt> to the system root, you can set it to a temporary location and create an installable binary package from it, and then install the binary package into the system root.

== Porting ==

The <tt>make</tt> command likely didn't succeed. You'll need to investigate the build errors and improve your standard library, or improve the package if it is unreasonable.

== Runtime ==

After you cross-compiled the package and installed it onto your operating system, it's time to try it out. If you implemented your operating system and standard library well, it should now work. Likely, especially if you made your own custom standard library, your first real ports won't. You now need to learn to debug programs you know nothing about on your new operating system. Have fun! Suddenly it'll work and you have something to talk about in the 'Aww, Yeah!' forum thread.

== Problems ==

Unfortunately, this is the real world and some packages misbehave in manners that breaks our naive cross-compilation. This is a bug in such packages; the practice described so far should be sufficient for well-made packages. Fortunately, community members likely know of such issues and generally work to upstream fixes, see the patch collections below. This section lists common problems and solutions.

In case it's a bug in these packages, it's advisable to patch the packages to fix the bug and upstream the bugfix (or report the issue), rather than poorly working around the issues in your build system (in ways that tend to get more and more complex). Generally, the best approach is to make the packages implement the interface we rely on above.

=== libtool .la files ===

Many libraries use libtool and install .la files into the system library directory. These work somewhat like pkg-config files, except often semantically wrong as they are not sysroot aware (but research the <tt>--with-sysroot</tt> option that libtool-aware configure scripts provide, it might save the system root path in the .la files which is wrong). The files can cause the build system to add library directories to the link command that weren't prefixed with the system root, causing the command link with the <tt>/usr/lib</tt> files from your local operating system.

It's entirely safe to delete these files, so set up your post-install build steps to delete any .la files on sight. The libraries generally install pkg-config files as well and the programs generally use pkg-config to locate libraries anyways (if not, it's perhaps a bug).

=== Dumb pkg-config use ===

Some packages simply invoke pkg-config with the raw <tt>pkg-config</tt> command, rather than using the <tt>PKG_CONFIG</tt> and <tt>PKG_CONFIG_FOR_BUILD</tt> variables. That is a bug. The correct logic is to use the variables if they exist and fall back on the raw command otherwise. The shell expression <tt>${PKG_CONFIG:=pkg-config}</tt> is useful as if it is set and non-empty, then it is used, otherwise pkg-config us used.

=== foo-config ===

Some packages (like libfreetype, libpng, libsdl, libxml2, and more) install a custom program like <tt>freetype-config</tt> into the bin directory. The idea is packages depending on libfreetype can run <tt>freetype-config --cflags --libs</tt> and it gets the compiler options needed to use the library. This scheme fails horribly in practice, as the bin directory of your operating system is not in your PATH (neither should it be!) and instead the <tt>freetype-config</tt> program of your distribution is used instead. The programs are not even sysroot-aware so it's not added in front of the compiler options. Suddenly your otherwise-nicely cross-compiled programs got linked with a Linux libfreetype with the Linux version of the headers.

The solution is to nuke these programs on sight. They're broken and should never be used or installed on your operating system. Tell the upstream developers to stop providing them and provide pkg-config files, and projects using them to use pkg-config instead. This is supposedly why pkg-config was created, to do this once and for all in a proper way. If you see programs using the foo-config programs, patch the configure script so they use pkg-config instead.

=== Running Cross-Compiled Programs ===

Some packages are not cross-compile aware and use the standard compiler (your cross-compiler) to build a local program that generates parts of the program and then runs the program. This doesn't work, it runs a cross-compiled executable and it won't run on your local operating system. The results vary from execve returning an error, infinite loops, or mysterious crashes. The bug is that the package should have used a variable like <tt>CC_FOR_BUILD</tt> to compile programs for the local system.

This occasionally also takes the form of configure tests that are mistakenly run (not just compiled and linked) when cross-compiling. A while ago, a lot of packages checked for Japanese locales by unconditionally running such tests when cross-compiling leading to fun results. The correct behavior is to attempt running this test, and if we are cross-compiling, then assume something reasonable (or delaying the test until the program is actually run).

=== Assuming the Worst ===

Some packages take portability to the levels where they want to support broken operating systems and do compatibility magic in these cases. This is often to work around a bug in a particular release of a particular operating system. This compatibility occasionally takes the form of a configure test that needs to be executed. In the event of cross-compilation, these tests needs to assume something. The developers made the error of assuming unknown systems are terribly broken, instead of just using a heuristic that FooBSD (before release 5) is broken and assuming the best about all other systems.

The correct solution is for packages to assume the best about unknown operating systems. This shifts the punishment from the good systems to the bad systems (so they break and they have the opportunity to fix bugs). This is not a problem for existing systems, because if the developers care about FooBSD, that's not an unknown system.

=== Gnulib ===

The GNU portability layer takes the form of a collection of files that everyone copies into their packages and then neglect to update often. These files are often deeply integrated into the package (i.e. hard to disable properly). The principle of replacing standard library functions if broken or missing is not too terribly bad - but Gnulib mixes it with a huge paranoia that the host system is terribly broken and assumes the very worst when cross-compiling. The result is that when you cross-compile these ports, huge amounts of compatibility code gets compiled in, and much of this compatibility code does not even work on unknown platforms. Of particular fun is code that needs to integrate deeply into stdio internals or when it replaces your printf. The result is that as you port packages, you often find yourself fixing the same gnulib code over and over (each time subtly different depending on when it was forked). When you improve your stdio implementation to be more standards-compliant, you find yourself needing to fix all those gnulib stdio-internals-aware files all over again, because some silly internal changed.

The solution is to scream in horror at how troublesome and unnecessary this scheme is as obviously you are capable of implementing a correct operating system. This racketeering scheme has <tt>#error</tt> statements that tell you to upstream preprocessor conditionals for your operating system, so they can relish in even more complexity that didn't need to exist in the first place.

[[User:sortie|Sortie]] has developed a [https://gitlab.com/sortix/sortix/wikis/Gnulib gnulib policy for his OS] that describes how to handle gnulib and it has a long list of secret autoconf variables that makes gnulib assume everything is perfect.

=== Custom Configure Script ===

Some packages considers autoconf to be hellish and refuses to use it. They occasionally replace an autoconf-generated configure script with a hand-written one. This is well, but they unfortunately often fail to completely implement the same interface and don't support cross-compilation. You can attempt to patch the configure script to it implements the same interface, or do whatever else gets the job done. You probably want to yell at the developers.

=== No DESTDIR ===

The package might not support the <tt>DESTDIR</tt> variable. You should patch the <tt>Makefile</tt> (perhaps <tt>Makefile.in</tt>) to support it and tell the upstream developers. Perhaps it's called something else like <tt>INSTALLROOT</tt>, you can rename it in the Makefile or add <tt>INSTALLROOT ?= $(DESTDIR)</tt>.

=== Packages Containing Dependencies ===

Sometimes packages contain copies of their dependencies as subdirectories. If you are really unlucky, they get used instead of the real deal. That means you have to port a two year old version all over again. If you are more unlucky, you get something that actually compiles on your operating system, but contains unfixed bugs and has security issues and this gets silently used. Extra fun is if they fixed bugs or enhanced the library and this is not in the upstream version. Tell upstream projects to get their act together if you see it, it's your responsibility to provide dependencies for a good reason.

=== Exotic problems ===

As you start porting packages that rarely get ported or cross-compiled, you'll likely start finding some seriously messed up packages. For instance, I once found a library that stores the path to the compiler and the compiler options in a header, so other libraries/programs depending on it could locate the compiler that way (and they did) - despite the fact they also had a full and generated configure script. At some point, you should reconsider whether this is actually software you want to port.

== Upstreaming Local Patches ==

As you port more packages, you gradually get a patch collection. Many of these patches likely works around issues in your system, but other patches solves general issues in the packages. Ideally, you should not need to patch packages at all. Though, your operating system is not terribly important to support in the upstream version and any patches will likely go stale soon as your operating system improves. However, the patches not related to your operating system is likely of value upstream (or to the other osdever's that also port stuff). You should send such patches upstream or file bug reports. In the long run, it decreases the size of your patch collection and makes it more maintainable.

== Patch Collections ==

It is probable that other community members have already ported a particular package. Their patches likely contain insight into how troublesome it is to do a port and what needs to be done. You can often find a collection of all their local patches in a central location.

* [[User:sortie|Sortie]] has a [https://pub.sortix.org/sortix/release/nightly/patches/ large collection of patches] and [https://gitlab.com/sortix/sortix/wikis/Ports notes on each port].

[[Category:Porting]]
````

<a id="w003629"></a>

## W003629 — Stan Dard

Bron-ID: W003629
Namespace: Main (0)
Revisie: 29280 | Datum: 2024-10-28T21:29:47Z
Bron: https://wiki.osdev.org/Stan_Dard
Revisie-URL: https://wiki.osdev.org/index.php?title=Stan_Dard&oldid=29280

````text
Stan knows how things should be. It's all nicely written down in standards. He sees how they fit together to form a beautiful system, well, if you disregard all the parts where the standard is stupid. He has a reasonably complete implementation of the areas he knows about and not-terribly-impressive code for the areas he is unskilled in. He spends much time rewriting code written by younger-and-unskilled instances of himself to satisfy his insatiable and ever-changing perfectionism. He is always ready to be helpfully unhelpful to newcomers by pointing out they should stop doing things their way and do it his way instead.

== Description == 

Stan suffers from an acute case of Not-Invented-Here. When he looks at the computing world today with all its horribleness and complexity, he wants to do better. Of course, he doesn't have the effort to rewrite everything (but perhaps he wants to, perhaps he ought to). He compromises by honoring standards as long as they are reasonable and goes with his gut otherwise. He understands how sometimes diverging from standards is the best thing to do, but there is always a price to pay for every divergence. He knows about the history of standards and why the standards are like they are.

If you give Stan sufficient time, he probably ends up with a mostly-complete implementation of POSIX (IEEE Std 1003.1, 2018 Edition). Of course, he started out inexperienced and naively made a bunch of bad design decisions that he now regrets. Much time is spent fixing the code made by younger instances of himself - he really should have been more ambitious than "I'm just having fun" and done things properly. As he gradually cleans up his act and learns the standards, his code gets cleaner and simpler, but he still isn't satisfied. On the other hand, he feels proud when he looks at the complexity of contemporary systems and likes how his solution is simpler even if currently suboptimal. He can usually split his code into two categories, the code that he is reasonably proud of, and the code that is scheduled for a rewrite. As such, more time is spent improving the operating system base than actually getting new features done.

That's certainly not all bad. His standard library is powerful, robustly made, and compatible with much third party software. His operating system has dozens of ports. Often his work is more like a Linux distribution preparing packages than a low-level hacker writing drivers. He ported a graphical user-interface toolkit, but has no functional display server (and no mouse driver). He has a port of git, but no networking. Somehow he even managed to port Python and Mono, though like most ports they're not really useful as he isn't actually using his operating system. He ported enough programs that his operating system is self-building, but not actually self-hosting as he still hasn't written that crucial harddisk driver.

Stan generally manages to do well in areas that he knows about and manages to neglect good work in areas outside his expertise. He strives to improve the way he does things, and stuck with a legacy codebase written by incompetent versions of himself, he often makes new recommendations that his code certainly doesn't follow.

Building his operating system is a fun experience involving getting yourself a modern Linux that implements the modern features his custom cross-tools expect, a version of the core shell utilities that supports the obscure long options he needs, building the highly customized binutils and gcc he maintains, bootstrapping his custom Make (as GNU Make is obviously not good enough), setting up his ports collection (highly patched to cross-compile properly), and so on. He deeply understands why this is the One True Way to cross-compile and his friendly documentation of the process scares established osdev community members.

== Pros & Cons ==
For Stan it's easy to port third party software, but it is sometimes hard, to be conformant to all the standards.

== Stan's Bookshelf ==

Stan generally has no bookshelf as the relevant standards can be accessed electronically or through his university library:

* The Intel and AMD CPU manuals
* The System V ABI
* The C standard (2011 revision)
* [http://pubs.opengroup.org/onlinepubs/9699919799/ POSIX.1-2008]
* Advanced Programming in the UNIX Environment
* Modern Operating Systems
* Practical Filesystem Design (with the Be File System)
* The usual RFC documents
* The Man Pages for his favorite Unix

== People and OSes related to Stan Dard ==

* [[User:Sortie|Sortie]]: ''This is really me.''
* [[User:Joeeagar|JoeEagar]]: ''Me too'' 
* [[User:Nunolava1998|NunoLava1998]]: ''I has transitioned from Duct Von Tape to Stan Dard.''
* [[User:Davidaylaian|DavidAylaian]]: ''I feel attacked; this is completely me. I have slowly moved from Duct Von Tape to Lino Commando, and now I have finally arrived here at Stan Dard.
* [[User:Techdude17|Techdude17]]: Random stumbled across this. As I continued reading, I realized "hey that's quite literally me"
* ''Insert your names here.''


[[Category:Developer Archetype]]
````

<a id="w003631"></a>

## W003631 — User:Veltas

Bron-ID: W003631
Namespace: User (2)
Revisie: 16691 | Datum: 2014-09-09T15:20:15Z
Bron: https://wiki.osdev.org/User:Veltas
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AVeltas&oldid=16691

````text
I am a complete novice in OS Dev, but may well end up contributing lots of minor changes to articles fixing obvious mistakes and anything I can rightly help with.
````

<a id="w003632"></a>

## W003632 — VirtIO

Bron-ID: W003632
Namespace: Main (0)
Revisie: 16692 | Datum: 2014-09-09T16:01:50Z
Bron: https://wiki.osdev.org/VirtIO
Revisie-URL: https://wiki.osdev.org/index.php?title=VirtIO&oldid=16692
Doorverwijzing: Virtio

````text
#REDIRECT [[virtio]]
````

<a id="w003633"></a>

## W003633 — JFFS2

Bron-ID: W003633
Namespace: Main (0)
Revisie: 30942 | Datum: 2026-09-04T15:46:18Z
Bron: https://wiki.osdev.org/JFFS2
Revisie-URL: https://wiki.osdev.org/index.php?title=JFFS2&oldid=30942

````text
{{stub}}{{Filesystems}}
Journalling Flash File System version 2 or JFFS2 is a filesystem (see [[File Systems]]) for use with flash memory devices. It is the successor to JFFS. JFFS2 has been included into the Linux kernel since September 23, 2001, when it was merged into the Linux kernel mainline as part of the kernel version 2.4.10 release. JFFS2 is also available for a few bootloaders, like Das U-Boot, Open Firmware, the eCos RTOS and the RedBoot. Most prominent usage of the JFFS2 comes from OpenWrt.
At least three file systems have been developed as JFFS2 replacements: LogFS, UBIFS, and [[YAFFS]].

----

JFFS2 introduced:

*Support for NAND flash devices. This involved a considerable amount of work as NAND devices have a sequential I/O interface and cannot be memory-mapped for reading.
*Hard links. This was not possible in JFFS because of limitations in the on-disk format.
*Compression. Four algorithms are available: zlib, rubin, rtime, and lzo.
*Better performance. JFFS treated the disk as a purely circular log. This generated a great deal of unnecessary I/O. The garbage collection algorithm in JFFS2 makes this mostly unnecessary.

[[Category:Filesystems]]
````

<a id="w003634"></a>

## W003634 — User:Bellezzasolo/He:Home

Bron-ID: W003634
Namespace: User (2)
Revisie: 16721 | Datum: 2014-09-15T18:31:32Z
Bron: https://wiki.osdev.org/User:Bellezzasolo/He:Home
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABellezzasolo%2FHe%3AHome&oldid=16721

````text
== שלי ==
ברוכים הבאים לאתר הבית שלי. מסיבה זו בבקשה לערוך את אזור הדיבורים, לא את החלק הזה
אני בודק תמיכה רב לשונית כיום
````

<a id="w003635"></a>

## W003635 — User talk:Ornitorrincos

Bron-ID: W003635
Namespace: User talk (3)
Revisie: 16728 | Datum: 2014-09-17T18:06:32Z
Bron: https://wiki.osdev.org/User_talk:Ornitorrincos
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AOrnitorrincos&oldid=16728

````text
Start of a UEFI bootloader using EDK II

<source lang="c">
#include <Uefi.h>
#include <Library/PcdLib.h>
#include <Library/UefiLib.h>
#include <Library/UefiApplicationEntryPoint.h>
#include <Protocol/SimpleFileSystem.h>


//
// UEFI entry point
// ImageHandle
// SystemTable - Pointer to the system table(provides runtime and boot services
//
// return EFI_SUCCESS on success
// otherwise consult error codes
//
EFI_STATUS
EFIAPI
UefiMain (
  IN EFI_HANDLE        ImageHandle,
  IN EFI_SYSTEM_TABLE  *SystemTable
  )
{
    // Pointers to the services tables
    EFI_BOOT_SERVICES * bootsrv;
    EFI_RUNTIME_SERVICES * runtimesrv;

    // Memory Map
    UINTN mapsize = 0;
    EFI_MEMORY_DESCRIPTOR * map = NULL;
    UINTN mapkey = 0;
    UINTN descriptorsize = 0;
    UINT32 version = 0;

    // INitialize the pointers
    bootsrv = SystemTable->BootServices;
    runtimesrv = SystemTable->RuntimeServices;

    // Obtain UEFI information
    Print(L"Firmware Vendor: %s Rev: 0x%08p\n", SystemTable->FirmwareVendor, SystemTable->FirmwareRevision);


    // It is going to be too small anyways
    if(bootsrv->GetMemoryMap(&mapsize, map, &mapkey, &descriptorsize, &version) == EFI_BUFFER_TOO_SMALL)
    {
        // we need to increase the size to allocate more than we need
        // as the following allocation will surely change the memory map
        mapsize += 1000;

        // Allocate memory for the memory map
        EFI_STATUS retalloc = bootsrv->AllocatePool(EfiBootServicesData, mapsize, (void**)&map);

        // check if we allocated
        if(retalloc != EFI_SUCCESS)
        {
            Print(L"Failed to allocate memory\n");
            if(retalloc == EFI_OUT_OF_RESOURCES)
            {
                Print(L"Out of resources\n");
            } else
            {
                Print(L"InvalidParameter(PoolType)\n");
            }

            Print(L"\n");
        }

        // we get the memory map with enough space to store it
        EFI_STATUS ret = bootsrv->GetMemoryMap(&mapsize, map, &mapkey, &descriptorsize, &version);

        // get total number of entries
        UINTN totalpages = mapsize/descriptorsize;

        Print(L"Descriptor size: %d\n", descriptorsize);
        Print(L"Total page tables: %d\n", totalpages);

        if(ret != EFI_SUCCESS)
        {
            Print(L"Failure with memory map\n");
            if(ret == EFI_BUFFER_TOO_SMALL)
            {
                Print(L"Buffer still too small\n");
            } else if(ret == EFI_INVALID_PARAMETER)
            {
                Print(L"Incorrect parameter\n");
            }
        } else
        {
            int i = 0;        // for loop index
            UINT64 accum = 0; // Accumulate total number of pages
            for(i = 0; i < totalpages; ++i)
            {

                // EFI_PHYSICAL_ADDRESS is actually a UINT64, saved this way to try to have compatibility
                EFI_PHYSICAL_ADDRESS intaddr = (EFI_PHYSICAL_ADDRESS)map;


                accum += map->NumberOfPages;
                map = (EFI_MEMORY_DESCRIPTOR*)(descriptorsize + intaddr); // who said integer pointer arithmetic was bad?
            }

            // each page holds 4KB
            accum *= 4;
            // Convert from KB to MB
            accum /= 1024;

            Print(L"Total Memory (MB): %d\n", accum);
        }

    }

    // This is test code to print available variables to the application
    // They seem to be vendor specific, so better not to relay on them
    /*{
        Print(L"\nVariables list:\n");

        CHAR16 text[4000];
        UINTN variablesize = 4000;
        EFI_GUID vendor;
        bootsrv->SetMem(text, 4000*2, 0);

        while(runtimesrv->GetNextVariableName(&variablesize, text, &vendor) == EFI_SUCCESS)
        {
            variablesize = 4000;
            Print(L"%s\n", text);
        }
    }*/

    // handles to the open files
    EFI_FILE_PROTOCOL * fileprot;     // this will open the current directory bu default(the one the application is in)
    EFI_FILE_PROTOCOL * kernelhandle; // handle to the kernel(each time we open a file we need a new handle)
    EFI_GUID guid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID; // to open the protocol

    // We try to open the protocol
    if(bootsrv->LocateProtocol(&guid, NULL, (void**)&fileprot) != EFI_SUCCESS)
      {
        Print(L"Failed File protocol\n");
        fileprot->Flush(fileprot);
      } else
      {
        // Now open the kernel binary(elf format)
        EFI_STATUS kernel_open = fileprot->Open(fileprot, &kernelhandle, L"kernel.bin", EFI_FILE_MODE_READ, EFI_FILE_READ_ONLY);

        if(kernel_open != EFI_SUCCESS)
          {
            // try to print the error type(not very concise)
            switch(kernel_open)
              {
              case EFI_NOT_FOUND:
                Print(L"kernel.bin not found\n");
                break;
              }
          } else
          {
            Print(L"Kernel Binary Found\n");
            // TODO call ExitBootServices
            // TODO actually read the file and jump into it
            // TODO setup all the necessary cruft for a 64 bit kernel
          }
      }

  return EFI_SUCCESS;
}

</source>
````

<a id="w003636"></a>

## W003636 — User:Sortie/Meaty Skeleton

Bron-ID: W003636
Namespace: User (2)
Revisie: 30812 | Datum: 2026-06-23T12:39:26Z
Bron: https://wiki.osdev.org/User:Sortie/Meaty_Skeleton
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie%2FMeaty_Skeleton&oldid=30812

````text
{{NoteBox|'''Note:''' This is a draft version of the proposed new Meaty Skeleton tutorial. It is being peer reviewed by the community.}}
{{FirstPerson}}
{{BeginnersWarning}}
{{Rating|1}}
{{Kernel designs}}

This tutorial continues from [[Bare Bones]] and creates a minimal template
operating system in the [[Stan Dard]] style suitable for further modification or
as inspiration for your initial operating system version. The [[Bare Bones]]
tutorial only gives you the absolutely minimal code to demonstrate how to
correctly cross-compile a kernel, however this is unsuitable as an example
operating system. Additionally, this tutorial implements necessary ABI features
needed to satisfy the ABI and compiler contracts to prevent possible mysterious
errors.

This tutorial also serves as the initial template tutorial on how to
[[Creating_a_C_Library|create your own libc]] (Standard C Library). The GCC
documentation explicitly states that libgcc requires the freestanding
environment to supply the <tt>memcmp</tt>, <tt>memcpy</tt>, <tt>memmove</tt>,
and <tt>memset</tt> functions, as well as <tt>abort</tt> on some platforms. We
will satisfy this requirement by creating a special kernel C library (libk) that
contains the parts of the user-space libc that are ''freestanding'' (doesn't
require any kernel features) as opposed to ''hosted'' libc features that need to
do system calls.

This tutorial implements a simple C kernel for <tt>i686-elf</tt> (32-bit
Intel), using a Multiboot 2 bootloader protocol, with a proper terminal driver
system with ANSI escape sequences that can output to the VGA text mode, to a
graphical buffer, or to the serial line. This tutorial has a full source code
example that you can learn from.

This tutorial has been peer reviewed by the osdev community to contain the best
practices.

== Preface ==

This tutorial is an example on how you could structure your operating system in
a manner that will continue to serve you well for the foreseeable future. This
tutorial serves as both inspiration and as an example for those that wish to something
different, while serving as a base for the rest. The tutorial does embed a few important
concepts into your operating system such as the existence of a libc, proper Unix
terminal semantics, as well as
indirectly other minor Unix and ABI semantics. Adapt what you wish from this
tutorial. Note that the Make-based build system constructed in
this tutorial is meant for Unix systems. There
is no need to make this tutorial portable across all operating systems as the
provided code is just an example.

We will name this new example operating system <tt>myos</tt>. This is just a
placeholder and you should replace all occurrences of <tt>myos</tt> with what you
decide to call your operating system.

First this tutorial will examine the design principles, then the example code of
MyOS will follow, and finally this tutorial will explore future directions.

== Bare Bones ==
{{Main|Bare Bones}}

You are expected to have completed the [[Bare Bones]] tutorial before continuing
to this tutorial. It is not strictly necessary to have completed [[Bare Bones]],
but doing so confirms that your development environment works as well as
explaining a number of core things.

You should probably discard the code you got from toying around with
[[Bare Bones]] and start over with this tutorial as your basis.

== Building a Cross-Compiler ==
:''Main article: [[GCC Cross-Compiler]], [[Why do I need a Cross Compiler?]]

You ''must'' use a [[GCC Cross-Compiler]] in this tutorial as in the
[[Bare Bones]] tutorial. You should use the '''i686-elf''' target in your
cross-compiler, though any <tt>ix86-elf</tt> target (but no less than i386) will
do fine for our purposes here.

You ''must'' configure your cross-binutils with the <tt>--with-sysroot</tt> option, otherwise linking will mysteriously fail with the ''this linker was not configured to use sysroots'' error message. If you forgot to configure your cross-binutils with that option, you'll have to rebuild it, but you can keep your cross-gcc.

You ''should'' configure your cross-gcc with the <tt>--enable-initfini-array</tt> option to use the new <tt>.init_array</tt> and <tt>.fini_array</tt> ABI for global constructors.

== Dependencies ==

You will need these dependencies in order to complete this tutorial:

* i686-elf toolchain, as discussed above.
* GRUB, for the grub-mkrescue command, along with the appropriate runtime files.
* Xorriso, the .iso creation engine used by grub-mkrescue.
* GNU make 4.0 or later.
* Qemu, optionally for testing the operating system.

This tutorial requires a GNU/Linux system, or a similar enough system. The BSD systems may almost work. macOS is not supported but can possibly be made to work with some changes. Windows is not supported, but Windows environments like Windows Subsystem For Linux (WSL) should work.

=== Debian-family Users ===

Install the i686-elf toolchain as described above and then install the packages <tt>xorriso grub-pc-bin</tt>.

== System Root ==

Normally when you compile programs for your local operating system, the compiler locates development files such as headers and libraries in system directories such as:

:<tt>/usr/include</tt>
:<tt>/usr/lib</tt>

These files are of course not usable for your operating system. Instead you want to have your own version of these directories that contains files for your operating system:

:<tt>/home/bwayne/myos/sysroot/usr/include</tt>
:<tt>/home/bwayne/myos/sysroot/usr/lib</tt>

The <tt>/home/bwayne/myos/sysroot</tt> directory acts as a fake root directory for your operating system. This is called a system root, or ''sysroot''.

You can think of the sysroot as the root directory for your operating system. Your build process will build each component of your operating system (kernel, standard library, programs) and gradually install them into the system root. Ultimately the system root will be a fully functional root filesystem for your operating system, you format a partition and copy the files there, add the appropriate configuration files, configure a bootloader to load the kernel from there, and use your harddisk driver and filesystem driver to read the files from there. The system root is thus a temporary directory that will ultimately become the actual root directory of your operating system.

In this example the cross system root is located as <tt>sysroot/</tt>, which is
a directory created by the build scripts and populated by the
<tt>make install</tt> targets. The makefiles will install the system headers
into the <tt>sysroot/usr/include</tt> directory, the system libraries into the
<tt>sysroot/usr/lib</tt> directory and the kernel itself into the
<tt>sysroot/boot</tt> directory.

We already use system roots because it will make it smoother to add a user-space when you get that far. This scheme is very convenient when you later [[Cross-Porting_Software|Port Third-Party Software by Cross-Compiling It]].

The <tt>-elf</tt> targets have no user-space and are incapable of having one. We configured the compiler with system root support, so it will look in <tt>${SYSROOT}/usr/lib</tt> as expected. We prevented the compiler from searching for a standard library using the --without-headers option when building <tt>i686-elf-gcc</tt>, so it will ''not'' look in <tt>${SYSROOT}/usr/include</tt>. (Once you add a user-space and a libc, you will configure your custom cross-gcc with <tt>--with-sysroot</tt> and it will look in <tt>${SYSROOT}/usr/include</tt>. As a temporary work-around until you get that far, we fix it by passing <tt>-isystem=/usr/include</tt>).

You can change the system root directory layout if you wish, but you will have to modify some Binutils and GCC source code and tell them [[OS Specific Toolchain|what your operating system is]]. This is advanced and not worth doing until you add a proper user-space. Note that the cross-linker currently looks in <tt>/lib</tt>, <tt>/usr/lib</tt> and <tt>/usr/local/lib</tt> by default, so you can move files there without changing Binutils. Also note that we use the <tt>-isystem</tt> option for GCC (as it was configured without a system include directory), so you can move that around freely.

== System Headers ==

The <tt>make sysroot-headers</tt> target simply installs the headers for your libc and
kernel (system headers) into <tt>sysroot/usr/include</tt>, but doesn't actually
cross-compile your operating system. This is useful as it allows you to
provide the compiler a copy of your headers before you actually compile your
system. You will [[Hosted_GCC_Cross-Compiler#Sysroot_Headers|need to provide the
standard library headers]] when you build a [[Hosted GCC Cross-Compiler]] in the 
future that is capable of a user-space.

Note how your cross-compiler comes with a number of fully freestanding headers
such as <tt>stddef.h</tt> and <tt>stdint.h</tt>. These headers simply declare
types and macros that are useful. Your kernel standard library will supply a
number of useful functions (such as <tt>strlen</tt>) that doesn't require system
calls and are freestanding except they need an implementation somewhere.

The libc and kernel headers cooperate in this example operating system, and work
together to provide the standard interfaces in accordance with the [[C|ISO C
and [[POSIX]] standards. The libc headers such as <tt>stdio.h</tt> may need to
include kernel headers to get important declarations that are ABI specific. In
this example OS, the kernel installs a header <tt>abi/winsize.h</tt> that
declares <tt>struct winsize</tt>. The kernel needs to know the structure for its
own internal uses, but it would later be exposed in the libc header
<tt>termios.h</tt> for the <tt>tcgetwinsize</tt> syscall. Other systems use e.g.
the <tt>bits/</tt> (glibc) or <tt>machine/</tt> directories for the internal ABI
headers. Software should never include those headers internally and you can
structure them as you please. Sortix for instance has <tt>__/foo.h</tt> headers
that declare important declarations from <tt>foo.h</tt> but prefixed with
<tt>__</tt>, which allows sharing these declarations without violating C and
POSIX namespace rules.

== Makefile Design ==

The top-level makefile iterates each module (libc and kernel) to build it and
install it into the sysroot. Every makefile includes the
<tt>build-aux/common.mk</tt> for shared logic about the toolchain and directory
structure.

To build MyOS:

<syntaxhighlight lang="bash">
make HOST=i686-elf
</syntaxhighlight>

Or to just build one module:

<syntaxhighlight lang="bash">
cd libc && make HOST=i686-elf SYSROOT=../sysroot
</syntaxhighlight>

The top-level Makefile sets the SYSROOT variable. You will have to override it
when building individual modules, otherwise the compiler will use its default
sysroot (which was disabled using <tt>--without-headers</tt> until you build
a [[OS Specific Toolchain]]).

The build system properly supports cross-compilation with a <tt>BUILD</tt>
system (the machine that you compile MyOS on, e.g. <tt>x86_64-linux-gnu</tt>)
and the HOST system (the machine that MyOS will run on, e.g. <tt>i686-elf</tt>).

By default, the build system does not cross-compile, and assumes the build
system is already running on MyOS. These semantics are how standard software
packages work. One day MyOS will be self-hosting and the default will feel
natural. As a counter-argument against cross-compilation by default, if two
architectures are added, e.g. <tt>i686-elf</tt> and <tt>x86_64-elf</tt>, then
the build system can't know which one to be the default cross-compilation host.

The makefiles in this example respect the environment variables (such as
<tt>CFLAGS</tt> that tell what default compile options are used to compile C
programs). This lets the user control stuff such as which optimization levels
are used, while a default is used if the user has no opinion. The makefiles also
make sure that particular options are always in <tt>CFLAGS</tt>. This is done by
having two phases in the makefiles: one that sets a default value and one that
adds mandatory options the project makefile requires:

<syntaxhighlight lang="make">
# Default CFLAGS:
CFLAGS?=-O2 -g

# Add mandatory options to CFLAGS:
CFLAGS:=$(CFLAGS) -Wall -Wextra
</syntaxhighlight>

The <tt>build-aux/common.mk</tt> file has special logic to override the default
<tt>CC</tt> value (since Make provides its own default value, which is not
desirable due to the HOST variable), while respecting the environment.

The makefiles use standard Make <tt>.SUFFIXES</tt> rules such as <tt>.c.o</tt>
to produce a <tt>.o</tt> file from a <tt>.c</tt> file, with special variables
that contain the inferred input and output files.

The <tt>-MD</tt> compiler option is used to generate makefile fragments with the
header dependencies of each object file. The makefile includes these fragments,
and make only recompiles the needed object files when source files and headers
are modified. The include statements use a leading hyphen, which means the make
continues if the fragments does not exist. The compiler crt files are copied
into the kernel directory to ensure the object dependency tracking is working
correctly, and to keep the kernel compilation commands simple.

== Architecture Directories ==

The projects in this example (libc and kernel) store all the architecture
dependent source files inside an <tt>arch/</tt> directory with their own
sub-makefile that has special configuration. This cleanly separates the systems
you support and will make it easier to port to other systems in the future.

The build system determines the <tt>HOST</tt> variable and places the name of
the architecture-specific directory into the <tt>HOSTARCH</tt> variable, from
which the <tt>arch.mk</tt> file is included with additional per-architecture
logic.

The <tt>boot.S</tt> <tt>_start</tt> assembly entry point will invoke the
<tt>kernel_early</tt> C function in the architecture-dependent <tt>i386.c</tt>
file. This file will perform the Multiboot 2 and i386 specific initialization
before passing control to the <tt>kernel_main</tt> function in the
platform-independent <tt>kernel.c</tt> file.

If <tt>kernel_early</tt> fails, it returns, which causes a <tt>hlt</tt> loop.
There is no panic function in myos yet, and even if there was, there is no safe
way to output information until a driver has been initialized for communication.

== Kernel Design ==

We have moved the kernel into its own directory named <tt>kernel/</tt>. It would
perhaps be better to call it something else if your kernel has another name than
your full operating system distribution, though calling it <tt>kernel/</tt>
makes it easier for other hobbyist developers to find the core parts of your new
operating system.

The kernel installs its public kernel headers into
<tt>sysroot/usr/include/kernel</tt>. This is useful if you decide to create a
kernel with modules, where modules can then simply include the public headers
from the main kernel. These are the private kernel headers, whereas
<tt>sysroot/usr/include/abi</tt> contains the public kernel headers in this
example.

The kernel implements the correct way of [[Calling_Global_Constructors|invoking
global constructors]] using the modern <tt>.init_array</tt> ABI (useful for C++
code and C code using <tt>__attribute__((constructor))</tt>. The bootstrap
assembly calls <tt>_init</tt> which lets the compiler run its own initialization
logic. This logic is likely empty in the new <tt>.init_array</tt> ABI, but it
may add additional logic if you build with various sanization options enabled,
and you are supposed to invoke it.
Afterwards <tt>boot.S</tt> iterates the <tt>.init_array</tt> section and invokes
each function pointer, which invokes all the global constructors.
This code could be done in C, but is intentionally done in assembly, to avoid a
theoretical chicken and egg problem where C code is technically only properly
defined after the runtime initialization is complete.
These constructors are invoked very early in the boot, and the constructor
attribute can ensure they are invoked in a particular order. You should only use
them to initialize global variables that could not be initialized at runtime,
but this early kernel environment can be dangerously non-initialized and should
be avoided and only used per careful rules.

Note how the global constructors require special linker script support.

The special <tt>__is_kernel</tt> macro lets the source code detect whether
it is part of the kernel.

== Multiboot2 ==

{{Main|Multiboot}}

GNU GRUB is used as the bootloader and the kernel uses the Multiboot2 bootloader
protocol. The Multiboot2 header must be in the first 32768 bytes of the
executable with 8-byte alignment. The linker script ensures the header is put
first in the myos executable. The invocation of the <tt>grub-file</tt> program
will fail the kernel build if it is not a valid Multiboot2 executable.

Multiboot2 improves on the original Multiboot specification with:

* Extensible tag system for requesting additional information actions from the bootloader, whereas Multiboot1 had hard-coded abilities.
* Support for the EFI pointer, which is required to access [[EFI|EFI Runtime Services]]. Without this ability, MyOS cannot install a bootloader into the EFI non-volatile memory variables.

Multiboot2 requires more complex structures. This example contains a minimal
version that has declarations needed for this tutorial only. See the
[https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html official Multiboot2 specification]
for the full set of available tags.

This example requests a framebuffer from the bootloader and locates the command
line that was given to the kernel by the bootloader configuration.

The provided multiboot tag search code locates the next tag after a tag. Each
tag starts on a 8-byte aligned address. The code aligns upwards by negating the
size, then aligning downwards with a simple bit mask, and then negating the
number again. This trick works due to the properties of unsigned integers, and
is very efficient.

== libc and libk Design ==

{{Main|Creating a C Library}}

The libc and libk are actually two versions of the same library, which is stored
in the directory <tt>libc/</tt>. The standard library is split into two
versions: freestanding and hosted. The difference is that the freestanding
library (libk) doesn't contain any of the code that only works in user-space,
such as system calls. The libk is also built with different compiler options,
just like the kernel isn't built like normal user-space code.

You are not required to have a libk. You could just as easily have a regular
libc and a fully separate minimal project inside the kernel directory. The libk
scheme avoids code duplication, so you don't have to maintain multiple versions
of <tt>strlen</tt> and such.

This example doesn't come with a usable libc. It could compile a libc.a that is
entirely useless, except being a skeleton we can build on when we add user-space
in a later tutorial. The libc archive is disabled until you're ready to turn it
on.

Note how the libc and libk split allows the functions such as
<tt>abort</tt> to share logic, but have different backends and implementations.
The libc variant would invoke a system call, whereas the libk variant would
directly invoke a function in the kernel.

Each standard function is put inside a file with the same name as the function
inside a directory with the name of the header. For instance, <tt>strlen</tt>
from <tt>string.h</tt> is in <tt>libc/string/strlen.c</tt> and <tt>stat</tt>
from <tt>sys/stat.h</tt> would be in <tt>libc/sys/stat/stat.c</tt>.

The standard headers use a BSD-like scheme where <tt>sys/cdefs.h</tt> declares
a bunch of useful preprocessor macros meant for internal use by the standard
library. All the function prototypes are wrapped in
<tt>extern "C" {</tt> and <tt>}</tt> such that C++ code can correctly
link against libc (as libc doesn't use C++ linkage). Note also how the compiler
provides the internal keyword __restrict unconditionally (even in C89 mode),
which is useful for adding the restrict keyword to function prototypes even when
compiling code in pre-C99 or C++ mode.

The special <tt>__is_libc</tt> macro lets the source code detect whether
it is part of the libc and <tt>__is_libk</tt> lets the source code detect
whether it's part of the libk binary.

This example comes with a small number of standard functions that serve as
examples and serve to satisfy ABI requirements. Note that the <tt>printf</tt>
function included is very minimal and intentionally doesn't handle most common
features.

== Font Rendering ==

The [[Bare Bones]] tutorial uses the VGA text mode. However, this legacy mode is
not always available on modern hardware, and is very limited.

This tutorial requests a 32-bit graphical framebuffer from the bootloader via a
multiboot2 tag. The bootloader will inform the kernel what mode was selected
in a multiboot2 tag, and it will load the appropriate driver.

A new graphical framebuffer screen driver <tt>fb</tt> implements font rendering
using the VGA font. It is possible to request the VGA font from the VGA
hardware, but it may not be available, and this example kernel instead has its
own copy of the font in <tt>vgafont.h</tt>. The font contains VGA glyphs per
Codepage 437, which overlaps with ASCII, but the character set is not Latin-1
nor unicode. Each glyph is 16 pixels tall and 9 pixels wide. The 9th column is
the same as the 8th column, meaning that each glyph requires 1 byte (8 bits) per
row, for a total of 16 bytes per glyph. This driver emulates the rendering of
the VGA text mode, so the drivers have the same capabilities, except the
<tt>fb</tt> driver is capable of 32-bit RGB graphics.

It is expensive to read from the framebuffer. For these reasons, the <tt>fb</tt>
driver keeps a grid of which characters have been rendered to each cell, which
is used to efficiently re-render when scrolling. Note how the bootloader can
select an arbitrary resolution, but the kernel does not implement dynamic memory
allocation. For these reasons, the screen is limited to a 256x128 hard-coded
window size, which is enough for a 2304x2048 pixel display. This limitation can
be lifted once memory allocation is implemented.

The <tt>vga</tt> driver supports just the VGA text mode, and is used if the
bootloader is unable to configure a graphical mode.

The font rendering used here does not implement the cursor. It is left as an
exercise for the reader. The VGA hardware will need to be programmed, and it
would need to be manually rendered on the framebuffer. Note the special case
where the cursor is at the edge of the screen, but a new line has not begun yet.

The font rendering is limited to 32-bit RGB graphics with a standard color
channel ordering. It is left as an exercise to the reader to support indexed
color palettes, 24-bit color channels, and other color channel orders.

== Terminal ==

{{Main|Terminals}}

This tutorial implements a properly structured Unix terminal system. The
semantics are subtle and important. The example here is complex, but the
structure is very intentional, and any proper Unix system will end up with this
structure. One can avoid a lot of problems and complexity by sticking to this
design.

Let's examine this deceptively simple line in <tt>kernel/kernel/kernel.c</tt>:

<syntaxhighlight lang="bash">
printf("Hello, \e[31mkernel \e[91;44mWorld\e[m!\n");
</syntaxhighlight>

'''<tt>printf</tt>''' is provided in libk and handles sequences such as %c and %s,
which makes it a powerful tool to format strings. Additional sequences such as
%d and %f will need to be implemented later in libc to provide all the standard
features. Each produced output character is sent to the <tt>putchar</tt>
function. In the future, <tt>printf</tt> will need to become a generic engine
with a backend such as <tt>vcbprintf</tt> to power the many standard variants of
the function.

'''<tt>putchar</tt>''' outputs a single byte. libk integrates with the kernel by
invoking the <tt>console_write</tt> function. The libc variant would do
something else in the future, such as invoking <tt>fputc</tt> to buffer the byte
on a stdio <tt>FILE</tt> stream.

'''<tt>console_write</tt>''' will write a byte to the kernel console using
<tt>tty_write</tt>. The kernel has a number of terminals, and one of them is
chosen during boot to be the kernel console, where important messages are
written.

'''<tt>tty_write</tt>''' performs terminal termios line discipline output
processing and invokes the <tt>transmit</tt> function pointer on the underlying
tty driver. The output processing is configured using the <tt>tcsetattr</tt>
system call on a real Unix system. Normally the <tt>c_oflag</tt> <tt>OPOST</tt>
and <tt>ONLCR</tt> bits are set, which translates the <tt>\n</tt> byte to the
<tt>\r\n</tt> sequence. Since these bits don't exist yet in MyOS, this
line discipline logic does the newline translation unconditionally. Note how the
<tt>\n</tt> byte only goes to the newline on a physical terminal device, but it
does not return the cursor to the beginning of the line. This is why the
<tt>\r</tt> carriage return character is required.

'''<tt>.transmit</tt>''' is invoked in the driver with an output byte.
The <tt>ttyS</tt> '''<tt>ttyS_transmit</tt>''' tty serial driver will directly write the byte
to the hardware register to transmit it on the serial line.
The <tt>vt</tt> '''<tt>vt_transmit</tt>''' tty driver instead implements a virtual terminal
connected to a screen driver, and maintains the window size, the cursor position,
and other terminal emulator state. Ordinary characters are rendered by invoking
the <tt>set_char</tt> method on the screen, while special sequences are sent
to the <tt>vt_escaped</tt> function.

'''<tt>vt_escaped</tt>''' processes the next byte in an ANSI escape sequence. These
sequences start with a <tt>\e</tt> <tt>(ESC)</tt> ASCII byte. The next byte is normally
a <tt>[</tt> byte, but other kinds of sequences exist. Additional numeric
parameters follow, each separated by a semicolon. The sequence ends with a
letter in the ASCII range <tt>@</tt> to <tt>~</tt>, after which
<tt>vt_run_ansi</tt> is invoked to execute the command.

'''<tt>vt_run_ansi</tt>''' executes the escape sequence after the parameters have
been parsed. For instance, <tt>\e[32;43m</tt> switches to font color 2 and
background color 3. The <tt>ansi_to_vga_color</tt> array is used to find the
appropriate VGA text mode color and the <tt>palette</tt> array is used
to find the appropriate RGB color. Many other standardized escape sequences
exist, see e.g. the venerable VT100 terminal and the many extension sets.

'''<tt>.set_char</tt>''' renders the character and its metadata (vga color, foreground
color, background color, additional attributes) using the screen driver. The
<tt>vga</tt> '''<tt>vga_set_char</tt>''' driver will simply copy the character to the
screen using the VGA memory. The <tt>fb</tt> '''<tt>fb_set_char</tt>''' driver will use the
emulated VGA font rendering (see above) to draw the character directly on the
framebuffer.

== Drivers ==

This example implements two tty drivers: <tt>vt</tt> for a local virtual
terminal and <tt>ttyS</tt> for a serial line terminal. The <tt>vt</tt> driver
is connected to a screen driver.

This example implements two screen drivers: <tt>vga</tt> for rendering text to
the VGA text mode, and <tt>fb</tt> for rendering text to a graphical
framebuffer.

The <tt>vt</tt> driver provides a single <tt>tty1</tt> device at this time,
which is connected to the appropriate screen driver, based on the graphics
settings selected by the bootloader.

The <tt>ttyS</tt> driver provides a single <tt>ttyS0</tt> device connected to
the first traditional serial line. A proper driver would do a lot more careful
initialization and the kernel command line would inform it about the baud
settings, the terminal window dimensions, and the possibly the initial TERM
environment variable value. This driver will work in virtual machines and
possibly on real hardware, depending on the initialization done by the firmware
and the bootloader. The driver uses [[Port IO|x86 Port IO]], which is outside
the scope of this tutorial.

The kernel selects which terminal is the kernel console based on its
<tt>--console=</tt> command line option. The <tt>tty1</tt> terminal is the
default. This way the kernel can be dynamically configured to output using
arbitrary drivers.

Drivers are implemented by declaring interfaces, which can have multiple
implementations. This example uses idiomatic C style driver interfaces as found
in other modern C kernels. For instance, <tt>struct tty</tt> starts with a pointer
to its <tt>struct tty_ops</tt> object, which contains the function pointers for
the driver. <tt>struct tty</tt> is the live instance object, and
<tt>struct screen_ops</tt> is the static method table.
This design is equivalent to but distinct from the vtable approach 
the [[C++]] programming language. The <tt>container_of</tt> macro is used to
safely downcast a pointer from a base class to the appropriate subclass, which
allows better refactoring as well as polymorphic multiple-inheritance
object-oriented programming.

== Paging and Memory Allocation ==

{{Main|Paging}}

The example in this tutorial does not have paging or memory allocation. Paging
is one of the most essential parts of a kernel and must be implemented early on
as one of the next steps. Paging must be enabled in order to enter 64-bit mode
on [[X86-64]].

A basic identity mapping of the kernel should be implemented in <tt>boot.S</tt>
in the future. Note how the multiboot2 structures could be located anywhere in
memory, and it is mandatory to carefully map those structures into memory as
they are explored. The multiboot2 information is linear in memory and its size
is stored in the first page, so it's easy to safely map it. Note also how the
graphical framebuffer could also be anywhere in memory, and also must be mapped
separately. This example simply avoids these concerns because paging is disabled
and the structures can be assumed to be identity mapped.

== Source Code ==

<!---

Greetings, editors! It's not desirable to have parts of this wiki not actually be on this wiki, e.g. a code dump hosted on another service. To mitigate this, the source code is hosted here primarily. It's inconvenient and error-prone to manually copy it, though, so I've set up a git repository that people can easily clone. If you make any changes to the article, send a merge request and/or ping me, I'll update the git repository ASAP and update the git revision hash below. If you wish to seize control (perhaps if I become unavailable) you can simply fork the git repository and update the links accordingly. --sortie

--->

You can easily download the source code using [[Git]] from the [https://gitlab.com/sortie/meaty-skeleton Meaty Skeleton Git repository]. This is preferable to doing a manual error-prone copy, as you may make a mistake or whitespace may get garbled due to bugs in our syntax highlighting. To clone the git repository, do:

<syntaxhighlight lang="bash">
git clone https://gitlab.com/sortie/meaty-skeleton.git -b new
</syntaxhighlight>

Check for differences between the git revision used in this article and what you cloned (empty output means there is no difference):
<syntaxhighlight lang="bash">
git diff e7b93d8c5ce5a110371bf004875d4e4fc896b31c..new
</syntaxhighlight>

Operating systems development is about being an expert. Take the time to read the code carefully through and understand it. Please seek further information and help if you don't understand aspects of it. This code is minimal and almost everything is done deliberately, often to pre-emptively solve future problems.
=== kernel ===

These files go into the kernel directory.

==== kernel/.gitignore ====

<syntaxhighlight>
myos
*.d
*.o
</syntaxhighlight>

==== kernel/Makefile ====

<syntaxhighlight lang="make">
# To cross-compile MyOS: make HOST=i386-elf SYSROOT=../sysroot
# By default the build system will assume it's running on MyOS.

include ../build-aux/common.mk

CFLAGS?=-O2 -g
CPPFLAGS?=
LDFLAGS?=
LIBS?=

CFLAGS:=$(CFLAGS) -ffreestanding -Wall -Wextra
CPPFLAGS:=$(CPPFLAGS) -D__is_kernel -Iinclude
LDFLAGS:=$(LDFLAGS)
LIBS:=$(LIBS) -nostdlib -lk -lgcc

ARCHDIR=arch/$(HOSTARCH)

-include $(ARCHDIR)/arch.mk

CFLAGS:=$(CFLAGS) $(KERNEL_ARCH_CFLAGS)
CPPFLAGS:=$(CPPFLAGS) $(KERNEL_ARCH_CPPFLAGS)
LDFLAGS:=$(LDFLAGS) $(KERNEL_ARCH_LDFLAGS)
LIBS:=$(LIBS) $(KERNEL_ARCH_LIBS)

KERNEL_OBJS=\
$(KERNEL_ARCH_OBJS) \
kernel/fb.o \
kernel/kernel.o \
kernel/tty.o \
kernel/vt.o \

OBJS=\
$(ARCHDIR)/crti.o \
$(ARCHDIR)/crtbegin.o \
$(KERNEL_OBJS) \
$(ARCHDIR)/crtend.o \
$(ARCHDIR)/crtn.o \

LINK_LIST=\
$(LDFLAGS) \
$(ARCHDIR)/crti.o \
$(ARCHDIR)/crtbegin.o \
$(KERNEL_OBJS) \
$(LIBS) \
$(ARCHDIR)/crtend.o \
$(ARCHDIR)/crtn.o \

.PHONY: all clean distclean install install-headers install-kernel
.SUFFIXES: .o .c .S

all: myos

myos: $(OBJS) $(ARCHDIR)/linker.ld
	$(CC) -T $(ARCHDIR)/linker.ld -o $@ $(CFLAGS) $(LINK_LIST)
	grub-file --is-x86-multiboot2 myos

$(ARCHDIR)/crtbegin.o $(ARCHDIR)/crtend.o:
	OBJ=`$(CC) $(CFLAGS) $(LDFLAGS) -print-file-name=$(@F)` && cp "$$OBJ" $@

.c.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(CFLAGS) $(CPPFLAGS)

.S.o:
	$(CC) -MD -c $< -o $@ $(CFLAGS) $(CPPFLAGS)

clean:
	rm -f myos
	find . -name "*.o" -delete
	find . -name "*.d" -delete

distclean: clean

install: install-headers install-kernel

install-headers:
	mkdir -p $(DESTDIR)$(INCLUDEDIR)
	cp -R --preserve=timestamps include/. $(DESTDIR)$(INCLUDEDIR)/.

install-kernel: myos
	mkdir -p $(DESTDIR)$(BOOTDIR)
	cp myos $(DESTDIR)$(BOOTDIR)

-include $(OBJS:.o=.d)
</syntaxhighlight>

==== kernel/arch/i386/arch.mk ====

<syntaxhighlight lang="make">
KERNEL_ARCH_CFLAGS=
KERNEL_ARCH_CPPFLAGS=
KERNEL_ARCH_LDFLAGS=
KERNEL_ARCH_LIBS=

KERNEL_ARCH_OBJS=\
$(ARCHDIR)/boot.o \
$(ARCHDIR)/i386.o \
$(ARCHDIR)/ttyS.o \
$(ARCHDIR)/vga.o \
</syntaxhighlight>

==== kernel/arch/i386/boot.S ====

<syntaxhighlight lang="asm">
# Declare Multiboot2 header.
.section .multiboot2
	.align 8
multiboot2:
	# The multiboot 2 header must be within the first 32768 bytes of the kernel,
	# and declares the architecture and an arbitrary amount of tags with
	# additional requests for the bootloader. The boot will fail if mandatory
	# tags are not supported by the bootloader.
	.set MAGIC, 0xE85250D6 # The bootloader searches for this magic number
	.set ARCHITECTURE, 0   # i386
	.long MAGIC
	.long ARCHITECTURE
	.long .Lmultiboot2_end - multiboot2 # Length
	.long -(MAGIC + ARCHITECTURE + .Lmultiboot2_end - multiboot2) # Checksum

	# Request the bootloader provide us with the following information in the
	# multiboot information structures.
.Lmbi_start:
	.align 8
	.short 1 # Multiboot2 information request tag
	.short 0x1 # Flags (mandatory tag)
	.long .Lmbi_end - .Lmbi_start # Size
	.long 1 # Boot command line
	.long 3 # Modules
	.long 6 # Memory map
	.long 8 # Framebuffer info
.Lmbi_end:

	# Request a 32-bit framebuffer if available.
	.align 8
	.short 5 # Framebuffer tag
	.short 0x0 # Flags (optional)
	.long 20 # Size
	.long 0 # Width (no preference)
	.long 0 # Height (no preference)
	.long 32 # Depth

	# Request that modules are page aligned and do not overlap with any other
	# information. This behavior makes it possible to reclaim those pages.
	.align 8
	.short 6 # Module alignment tag
	.short 0x1 # Flags (mandatory tag)
	.long 8 # Size

	# End of multiboot tags.
	.align 8
	.short 0 # End tag
	.short 0x0 # Flags (optional tag)
	.long 8 # Size
.size multiboot2, . - multiboot2
.Lmultiboot2_end:

# Reserve a stack for the initial thread.
.section .bss
	.align 16
stack_bottom:
	.skip 16384 # 16 KiB
stack_top:

# The kernel entry point.
.section .text
.global _start
.type _start, @function
_start:
	# Use the stack for the initial thread.
	movl $stack_top, %esp

	# Prepare parameters for kernel_early.
	subl $8, %esp # 16-byte align stack for call
	push %ebx # Multiboot2 pointer (to kernel_early)
	push %eax # Multiboot2 magic (to kernel_early)

	# Run the compiler initialization code.
	call _init

	# Run the global constructors by executing the init array function pointers.
	mov $__init_array_start, %ebx
.Lnext:
	cmp $__init_array_end, %ebx
	jae .Ldone
	call *(%ebx)
	add $4, %ebx
	jmp .Lnext
.Ldone:

	# Execute the kernel.
	call kernel_early

	# Hang if the kernel returns.
	cli
1:	hlt
	jmp 1b
.size _start, . - _start

.section bss
	.align 4
	.weak __init_array_end
	.weak __init_array_start
	.hidden __init_array_end
	.hidden __init_array_start
</syntaxhighlight>

==== kernel/arch/i386/crti.S ====

<syntaxhighlight lang="asm">
.section .init
.global _init
.type _init, @function
_init:
	push %ebp
	movl %esp, %ebp
	/* gcc will nicely put the contents of crtbegin.o's .init section here. */

.section .fini
.global _fini
.type _fini, @function
_fini:
	push %ebp
	movl %esp, %ebp
	/* gcc will nicely put the contents of crtbegin.o's .fini section here. */
</syntaxhighlight>

==== kernel/arch/i386/crtn.S ====

<syntaxhighlight lang="asm">
.section .init
	/* gcc will nicely put the contents of crtend.o's .init section here. */
	popl %ebp
	ret

.section .fini
	/* gcc will nicely put the contents of crtend.o's .fini section here. */
	popl %ebp
	ret
</syntaxhighlight>

==== kernel/arch/i386/i386.c ====

<syntaxhighlight lang="c">
#include <stdint.h>
#include <string.h>

#include <kernel/fb.h>
#include <kernel/kernel.h>
#include <kernel/screen.h>
#include <kernel/tty.h>
#include <kernel/vt.h>

#include "multiboot2.h"
#include "ttyS.h"
#include "vga.h"

struct fb fb;
struct vga vga;
struct ttyS ttyS1_tty;
struct vt tty1_tty;

void kernel_early(uint32_t magic, struct multiboot2_info* multiboot2) {
	// Verify the multiboot2 magic number.
	if (magic != MULTIBOOT2_BOOTLOADER_MAGIC)
		return;

	// Locate the kernel command line options.
	struct multiboot2_tag_string* cmdline_tag = (struct multiboot2_tag_string*)
		multiboot2_tag_lookup(multiboot2, MULTIBOOT2_TAG_TYPE_CMDLINE);
	const char* cmdline = cmdline_tag ? cmdline_tag->string : "";

	// Create a /dev/ttyS0 device for the first serial line.
	// The appropriate settings for each serial port will vary, and the command
	// line should be able to inform the kernel about the port, the baud
	// settings, the window dimensions, the TERM variable, and so on. For now,
	// assume traditional values.
	struct tty* ttyS0 = ttyS_init(&ttyS1_tty, 0x3F8, 80, 25);

	// Try to locate a multiboot2 tag with framebuffer information.
	struct multiboot2_tag_framebuffer* fbinfo =
		(struct multiboot2_tag_framebuffer*)
		multiboot2_tag_lookup(multiboot2, MULTIBOOT2_TAG_TYPE_FRAMEBUFFER);
	struct multiboot2_tag_framebuffer_common* fb_common =
		fbinfo ? &fbinfo->common : NULL;

	// Initialize a screen for tty1 on the framebuffer.
	struct screen* tty1_screen;
	if (!fb_common ||
	    fb_common->framebuffer_type == MULTIBOOT2_FRAMEBUFFER_TYPE_EGA_TEXT) {
		// Use the VGA text mode.
		tty1_screen = vga_init(&vga);
	} else if (fb_common->framebuffer_type == MULTIBOOT2_FRAMEBUFFER_TYPE_RGB) {
		// Use a RGB framebuffer.
		// Verify the framebuffer is 32-bit.
		if (fb_common->framebuffer_bpp != 32)
			return;
		tty1_screen = fb_init(
			&fb, fb_common->framebuffer_addr,
			fb_common->framebuffer_width, fb_common->framebuffer_height,
			fb_common->framebuffer_pitch, fb_common->framebuffer_bpp);
	} else {
		// Unsupported framebuffer type.
		return;
	}

	// Create a /dev/tty1 device connected to the screen.
	struct tty* tty1 = vt_init(&tty1_tty, tty1_screen);

	// Select the appropriate terminal device as the kernel /dev/console.
	if (!strcmp(cmdline, "--console=ttyS0")) {
		dev_console = ttyS0;
	} else { // --console=tty1 is default
		dev_console = tty1;
	}

	// Run the main kernel.
	kernel_main();
}
</syntaxhighlight>

==== kernel/arch/i386/ioport.h ====

<syntaxhighlight lang="c">
#ifndef ARCH_I386_IOPORT_H
#define ARCH_I386_IOPORT_H

#include <stdint.h>

__attribute__((unused))
static inline uint8_t outport8(uint16_t port, uint8_t value)
{
	asm volatile ("outb %1, %0" : : "dN" (port), "a" (value));
	return value;
}

__attribute__((unused))
static inline uint8_t inport8(uint16_t port)
{
	uint8_t result;
	asm volatile("inb %1, %0" : "=a" (result) : "dN" (port));
	return result;
}

#endif
</syntaxhighlight>

==== kernel/arch/i386/linker.ld ====

<syntaxhighlight>
/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* It used to be universally recommended to use 1M as a start offset,
	   as it was effectively guaranteed to be available under BIOS systems.
	   However, UEFI has made things more complicated, and experimental data
	   strongly suggests that 2M is a safer place to load. In 2016, a new
	   feature was introduced to the multiboot2 spec to inform bootloaders
	   that a kernel can be loaded anywhere within a range of addresses and
	   will be able to relocate itself to run from such a loader-selected
	   address, in order to give the loader freedom in selecting a span of
	   memory which is verified to be available by the firmware, in order to
	   work around this issue. This does not use that feature, so 2M was
	   chosen as a safer option than the traditional 1M. */
	. = 2M;

	/* First put the multiboot2 header, as it is required to be put in the first
	   32768 bytes in the image or the bootloader won't recognize the format.
	   After we'll put the .text section. */
	.text BLOCK(4K) : ALIGN(4K)
	{
		*(.multiboot2)
		*(.text)
	}

	/* Read-only data. */
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.rodata)
	}

	/* Read-write data (initialized) */
	.data BLOCK(4K) : ALIGN(4K)
	{
		*(.data)
	}

	/* Read-write data (uninitialized) and stack */
	.bss BLOCK(4K) : ALIGN(4K)
	{
		*(COMMON)
		*(.bss)
	}

	/* Modern global constructors */
	.init_array :
	{
		PROVIDE_HIDDEN (__init_array_start = .);
		KEEP (*(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)))
		KEEP (*(.init_array EXCLUDE_FILE (*crtbegin.o *crtbegin?.o *crtend.o *crtend?.o ) .ctors))
		PROVIDE_HIDDEN (__init_array_end = .);
	}

	/* Modern global destructors */
	.fini_array :
	{
		PROVIDE_HIDDEN (__fini_array_start = .);
		KEEP (*(SORT_BY_INIT_PRIORITY(.fini_array.*) SORT_BY_INIT_PRIORITY(.dtors.*)))
		KEEP (*(.fini_array EXCLUDE_FILE (*crtbegin.o *crtbegin?.o *crtend.o *crtend?.o ) .dtors))
		PROVIDE_HIDDEN (__fini_array_end = .);
	}

	/* Compiler process initialization */
	.ctors :
	{
		KEEP (*crtbegin.o(.ctors))
		KEEP (*crtbegin?.o(.ctors))
		KEEP (*(EXCLUDE_FILE (*crtend.o *crtend?.o ) .ctors))
		KEEP (*(SORT(.ctors.*)))
		KEEP (*(.ctors))
	}

	/* Compiler process finalization */
	.dtors :
	{
		KEEP (*crtbegin.o(.dtors))
		KEEP (*crtbegin?.o(.dtors))
		KEEP (*(EXCLUDE_FILE (*crtend.o *crtend?.o ) .dtors))
		KEEP (*(SORT(.dtors.*)))
		KEEP (*(.dtors))
	}

	/* The compiler may produce other sections, put them in the proper place in
	   in this file, if you'd like to include them in the final kernel. */
}
</syntaxhighlight>

==== kernel/arch/i386/multiboot2.h ====

<syntaxhighlight lang="c">
#ifndef ARCH_I386_MULTIBOOT2_H
#define ARCH_I386_MULTIBOOT2_H

#include <stdint.h>

#define MULTIBOOT2_BOOTLOADER_MAGIC 0x36d76289

struct multiboot2_info {
	uint32_t total_size;
	uint32_t reserved;
};

struct multiboot2_tag {
	uint32_t type;
	uint32_t size;
};

#define MULTIBOOT2_TAG_TYPE_CMDLINE 1

struct multiboot2_tag_string {
	uint32_t type;
	uint32_t size;
	char string[];
};

#define MULTIBOOT2_TAG_TYPE_FRAMEBUFFER 8

#define MULTIBOOT2_FRAMEBUFFER_TYPE_INDEXED 0
#define MULTIBOOT2_FRAMEBUFFER_TYPE_RGB 1
#define MULTIBOOT2_FRAMEBUFFER_TYPE_EGA_TEXT 2

struct multiboot2_tag_framebuffer_common {
	uint32_t type;
	uint32_t size;
	uint64_t framebuffer_addr;
	uint32_t framebuffer_pitch;
	uint32_t framebuffer_width;
	uint32_t framebuffer_height;
	uint8_t framebuffer_bpp;
	uint8_t framebuffer_type;
	uint16_t reserved;
};

struct multiboot2_color {
	uint8_t red;
	uint8_t green;
	uint8_t blue;
};

struct multiboot2_tag_framebuffer {
	struct multiboot2_tag_framebuffer_common common;
	union {
		struct {
			uint16_t framebuffer_palette_num_colors;
			struct multiboot2_color framebuffer_palette[0];
		};
		struct {
			uint8_t framebuffer_red_field_position;
			uint8_t framebuffer_red_mask_size;
			uint8_t framebuffer_green_field_position;
			uint8_t framebuffer_green_mask_size;
			uint8_t framebuffer_blue_field_position;
			uint8_t framebuffer_blue_mask_size;
		};
	};
};

// Find the first tag in the multiboot2 information.
static inline
struct multiboot2_tag* multiboot2_tag_begin(struct multiboot2_info* info) {
	uintptr_t ptr = (uintptr_t) info;
	ptr += sizeof(*info);
	ptr = -(-ptr & ~7UL); // Align upwards: Negate, align downwards, negate.
	struct multiboot2_tag* tag = (struct multiboot2_tag*) ptr;
	return tag->type == 0 ? NULL : tag;
}

// Find the next tag following an existing tag.
static inline
struct multiboot2_tag* multiboot2_tag_next(struct multiboot2_tag* tag) {
	uintptr_t ptr = (uintptr_t) tag;
	ptr += tag->size;
	ptr = -(-ptr & ~7UL);
	tag = (struct multiboot2_tag*) ptr;
	return tag->type == 0 ? NULL : tag;
}

// Search through the multiboot2 tags for a tag with the desired type.
static inline
struct multiboot2_tag* multiboot2_tag_lookup(struct multiboot2_info* info,
                                             uint32_t type) {
	struct multiboot2_tag* tag = multiboot2_tag_begin(info);
	while (tag) {
		if (tag->type == type)
			return tag;
		tag = multiboot2_tag_next(tag);
	}
	return NULL;
}

#endif
</syntaxhighlight>

==== kernel/arch/i386/ttyS.c ====

<syntaxhighlight lang="c">
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <kernel/kernel.h>
#include <kernel/tty.h>

#include "ioport.h"
#include "ttyS.h"

// Check whether the serial port is ready to transmit another byte.
static bool serial_can_transmit(uint16_t port) {
	return inport8(port + 5) & 0x20;
}

// Transmit a byte on a serial port when it becomes possible to transmit.
void serial_write(uint16_t port, unsigned char byte) {
	while (!serial_can_transmit(port)) {
	}
	outport8(port, byte);
}

// Initialize a ttyS object connected to a console driver.
struct tty* ttyS_init(struct ttyS* self, uint16_t port, unsigned short width,
                      unsigned short height) {
	memset(self, 0, sizeof(*self));
	self->base.ops = &ttyS_ops;
	self->port = port;
	self->width = width;
	self->height = height;
	// Real hardware may require real initialization of settings such as baud.
	return &self->base;
}

// Process the next output byte.
static bool ttyS_transmit(struct tty* base, unsigned char uc) {
	struct ttyS* self = container_of(base, struct ttyS, base);
	serial_write(self->port, uc);
	return true;
}

// Get the terminal window size.
static int ttyS_tcgetwinsize(struct tty* base, struct winsize* ws) {
	struct ttyS* self = container_of(base, struct ttyS, base);
	ws->ws_col = self->width;
	ws->ws_row = self->height;
	return 0;
}

const struct tty_ops ttyS_ops = {
	.transmit = ttyS_transmit,
	.tcgetwinsize = ttyS_tcgetwinsize,
};
</syntaxhighlight>

==== kernel/arch/i386/ttyS.h ====

<syntaxhighlight lang="c">
#ifndef ARCH_I386_TTYS_H
#define ARCH_I386_TTYS_H

#include <stdint.h>

#include <kernel/tty.h>

void serial_write(uint16_t port, unsigned char byte);

// A ttyS device implements a tty connected to a hardware serial line.
struct ttyS {
	struct tty base;
	uint16_t port;
	unsigned short width;
	unsigned short height;
};

extern const struct tty_ops ttyS_ops;

struct tty* ttyS_init(struct ttyS* self, uint16_t port, unsigned short width,
                      unsigned short height);

#endif
</syntaxhighlight>

==== kernel/arch/i386/vga.c ====

<syntaxhighlight lang="c">
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <kernel/screen.h>
#include <kernel/kernel.h>
#include <kernel/vga.h>

#include "vga.h"

static uint16_t* const VGA_MEMORY = (uint16_t*) 0xB8000;
static const size_t VGA_WIDTH = 80;
static const size_t VGA_HEIGHT = 25;

// Format a VGA text mode entry with the console character and color.
static inline uint16_t vga_entry_from_sc(const struct screen_char* sc) {
	return vga_entry(sc->uc, sc->vgacolor);
}

// Initialize a VGA console.
struct screen* vga_init(struct vga* self) {
	memset(self, 0, sizeof(*self));
	self->base.ops = &vga_ops;
	self->memory = VGA_MEMORY;
	self->width = VGA_WIDTH;
	self->height = VGA_HEIGHT;
	return &self->base;
}

// Get the window size.
static void vga_tcgetwinsize(struct screen* base, struct winsize* ws) {
	struct vga* self = container_of(base, struct vga, base);
	ws->ws_col = self->width;
	ws->ws_row = self->height;
}

// Clear the console.
static void vga_clear(struct screen* base, const struct screen_char* fill) {
	struct vga* self = container_of(base, struct vga, base);
	uint16_t entry = vga_entry_from_sc(fill);
	for (unsigned short y = 0; y < self->height; y++) {
		for (unsigned short x = 0; x < self->width; x++) {
			size_t index = y * self->width + x;
			self->memory[index] = entry;
		}
	}
}

// Set a character on the console.
static void vga_set_char(struct screen* base, const struct screen_char* sc,
                                 unsigned short x, unsigned short y) {
	struct vga* self = container_of(base, struct vga, base);
	size_t index = y * self->width + x;
	uint16_t entry = vga_entry_from_sc(sc);
	self->memory[index] = entry;
}

// Scroll the console one line.
static void vga_scroll(struct screen* base, const struct screen_char* fill) {
	struct vga* self = container_of(base, struct vga, base);
	for (unsigned short y = 0; y < self->height - 1; y++) {
		for (unsigned short x = 0; x < self->width; x++) {
			size_t to = y * self->width + x;
			size_t from = (y + 1) * self->width + x;
			self->memory[to] = self->memory[from];
		}
	}
	uint16_t entry = vga_entry_from_sc(fill);
	for (unsigned short x = 0; x < self->width; x++) {
		unsigned short y = self->height - 1;
		size_t index = y * self->width + x;
		self->memory[index] = entry;
	}
}

const struct screen_ops vga_ops = {
	.tcgetwinsize = vga_tcgetwinsize,
	.clear = vga_clear,
	.set_char = vga_set_char,
	.scroll = vga_scroll,
};
</syntaxhighlight>

==== kernel/arch/i386/vga.h ====

<syntaxhighlight lang="c">
#ifndef ARCH_I386_VGA_H
#define ARCH_I386_VGA_H

#include <stdint.h>

#include <kernel/screen.h>

struct vga {
	struct screen base;
	uint16_t* memory;
	unsigned short width;
	unsigned short height;
};

extern const struct screen_ops vga_ops;

struct screen* vga_init(struct vga* self);

#endif
</syntaxhighlight>

==== kernel/include/abi/winsize.h ====

<syntaxhighlight lang="c">
#ifndef _ABI_WINSIZE_H
#define _ABI_WINSIZE_H

struct winsize {
	unsigned short ws_row;
	unsigned short ws_col;
};

#endif
</syntaxhighlight>

==== kernel/include/kernel/fb.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_FB_H
#define _KERNEL_FB_H

#include <stdint.h>

#include <kernel/screen.h>

struct fb {
	struct screen base;
	uint64_t addr;
	uint32_t width;
	uint32_t height;
	uint32_t pitch;
	uint32_t bpp;
	uint32_t* buffer;
	unsigned short columns;
	unsigned short rows;
	struct screen_char* grid;
};

extern const struct screen_ops fb_ops;

struct screen* fb_init(struct fb* self, uint64_t addr, uint32_t width,
                       uint32_t height, uint32_t pitch, uint32_t bpp);

#endif
</syntaxhighlight>

==== kernel/include/kernel/kernel.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_KERNEL_H
#define _KERNEL_KERNEL_H

#include <stddef.h>

#define container_of(ptr, type, member) \
	((type*) ((char*) (ptr) - offsetof(type, member)))

void kernel_main(void);

#endif
</syntaxhighlight>

==== kernel/include/kernel/screen.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_SCREEN_H
#define _KERNEL_SCREEN_H

#include <stddef.h>
#include <stdint.h>

#include <abi/winsize.h>

// A generic screen character with attributes and colors, which can be
// rendered with the best approximation possible by the console drivers.
struct screen_char {
	unsigned char uc;
	uint8_t vgacolor;
	uint16_t attr;
	uint32_t fg;
	uint32_t bg;
};

// Base structure for the screen interface.
struct screen {
	const struct screen_ops* ops;
};

// A console provides a two-dimensional grid of characters that can be updated
// and re-rendered by drivers using these efficient functions.
struct screen_ops {
	void (*tcgetwinsize)(struct screen* self, struct winsize* ws);
	void (*clear)(struct screen* self, const struct screen_char* fill);
	void (*set_char)(struct screen* self, const struct screen_char* sc,
	                 unsigned short x, unsigned short y);
	void (*scroll)(struct screen* self, const struct screen_char* fill);
};

#endif
</syntaxhighlight>

==== kernel/include/kernel/tty.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_TTY_H
#define _KERNEL_TTY_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <abi/winsize.h>

// Base structure for the tty interface.
struct tty {
	const struct tty_ops* ops;
};

// A tty is a terminal implemented by a driver.
//
// Terminals including the tty_console, which graphically renders to a local
// console framebuffer, but can also be a serial driver that communicates with
// a hardware device subject to line discipline, or a pty pseudoterminal
// entirely in software.
struct tty_ops {
	bool (*transmit)(struct tty* self, unsigned char uc);
	int (*tcgetwinsize)(struct tty* self, struct winsize* ws);
};

size_t tty_write(struct tty* self, const void* buf, size_t len);

extern struct tty* dev_console;

size_t console_write(const void* data, size_t size);
int console_tcgetwinsize(struct winsize* ws);

#endif
</syntaxhighlight>

==== kernel/include/kernel/vga.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_VGA_H
#define _KERNEL_VGA_H

#include <stdint.h>

enum vga_color {
	VGA_COLOR_BLACK = 0,
	VGA_COLOR_BLUE = 1,
	VGA_COLOR_GREEN = 2,
	VGA_COLOR_CYAN = 3,
	VGA_COLOR_RED = 4,
	VGA_COLOR_MAGENTA = 5,
	VGA_COLOR_BROWN = 6,
	VGA_COLOR_LIGHT_GREY = 7,
	VGA_COLOR_DARK_GREY = 8,
	VGA_COLOR_LIGHT_BLUE = 9,
	VGA_COLOR_LIGHT_GREEN = 10,
	VGA_COLOR_LIGHT_CYAN = 11,
	VGA_COLOR_LIGHT_RED = 12,
	VGA_COLOR_LIGHT_MAGENTA = 13,
	VGA_COLOR_LIGHT_BROWN = 14,
	VGA_COLOR_WHITE = 15,
};

static inline uint8_t vga_entry_color(enum vga_color fg, enum vga_color bg) {
	return fg | bg << 4;
}

static inline uint16_t vga_entry(unsigned char uc, uint8_t color) {
	return (uint16_t) uc | (uint16_t) color << 8;
}

#endif
</syntaxhighlight>

==== kernel/include/kernel/vt.h ====

<syntaxhighlight lang="c">
#ifndef _KERNEL_VT_H
#define _KERNEL_VT_H

#include <stdint.h>

#include <kernel/tty.h>

enum ansi_state {
	ANSI_STATE_NONE,
	ANSI_STATE_CSI,
	ANSI_STATE_COMMAND,
};

#define ANSI_PARAM_MAX 16

// A vt (virtual terminal) implements a tty connected to a screen.
//
// It handles escape code processing and allows efficient re-renders using an
// arbitrary screen driver.
struct vt {
	struct tty base;
	unsigned short row;
	unsigned short column;
	unsigned short width;
	unsigned short height;
	uint8_t vgacolor;
	uint16_t attr;
	uint32_t fgcolor;
	uint32_t bgcolor;
	struct screen* screen;
	enum ansi_state ansi_state;
	unsigned int ansi_params[ANSI_PARAM_MAX];
	size_t ansi_count;
};

extern const struct tty_ops vt_ops;

struct tty* vt_init(struct vt* self, struct screen* screen);

#endif
</syntaxhighlight>

==== kernel/kernel/fb.c ====

<syntaxhighlight lang="c">
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <kernel/fb.h>
#include <kernel/kernel.h>
#include <kernel/screen.h>

#define FONT_REALWIDTH 8
#define FONT_WIDTH 9
#define FONT_HEIGHT 16
#define FONT_CHARSIZE (FONT_REALWIDTH * FONT_HEIGHT / 8)
#define FONT_NUMCHARS 256

#define MAX_COLUMNS 256
#define MAX_ROWS 128

#include "vgafont.h"

// Initialize a framebuffer console.
struct screen* fb_init(struct fb* self, uint64_t addr, uint32_t width,
                       uint32_t height, uint32_t pitch, uint32_t bpp) {
	memset(self, 0, sizeof(*self));
	self->base.ops = &fb_ops;
	self->addr = addr;
	self->width = width;
	self->height = height;
	self->pitch = pitch;
	self->bpp = bpp;
	// Paging is not implemented. addr is a 64-bit physical address. For now,
	// simply truncate it to pointer size and reinterpret it as a pointer.
	self->buffer = (uint32_t*) (uintptr_t) addr;
	self->columns = width / 9;
	self->rows = height / 16;
	// Dynamic memory allocation is not implemented. For now, simply hard-code
	// a reasonably large grid of console characters on the screen, which is
	// used to redraw without expensive reads from the framebuffer. This only
	// works once because it's a global variable.
	if (MAX_COLUMNS < self->columns)
		self->columns = MAX_COLUMNS;
	if (MAX_ROWS < self->rows)
		self->rows = MAX_ROWS;
	static struct screen_char static_grid[MAX_COLUMNS * MAX_ROWS];
	self->grid = static_grid;
	return &self->base;
}

// Render a character using the hard-coded VGA font. It is possible to query
// the VGA hardware for the font, but this solution ensures a font is always
// available, regardless of the hardware.
static void fb_render(struct fb* self, unsigned short col, unsigned short row) {
	// The VGA font uses 16 pixel tall and 9 pixel wide characters, but the 9th
	// column repeats the 8th column. This way each row is one byte (8 bits).
	// Each character takes 16 bytes in the font. The font contains 256
	// characters numbered per Code Page 437, which matches ASCII for the
	// printable characters. There is no unicode support right now.
	// For Unicode support, one would switch struct screen_char to wchar_t, do
	// UTF-8 decoding in tty_console, translate the unicode codepoint here to
	// the best CP437 approximation, and otherwise render a U+FFFD codepoint.
	const struct screen_char* sc = &self->grid[row * MAX_COLUMNS + col];
	const uint8_t* charfont = vgafont + 16 * sc->uc;
	for (size_t y = 0; y < FONT_HEIGHT; y++) {
		size_t buffer_pitch = self->pitch / sizeof(self->buffer[0]);
		uint32_t* data = self->buffer +
		                 buffer_pitch * (FONT_HEIGHT * row + y) +
		                 col * FONT_WIDTH;
		uint8_t line_bitmap = charfont[y];
		for (size_t x = 0; x < FONT_REALWIDTH; x++)
			data[x] = line_bitmap & 1U << (7 - x) ? sc->fg : sc->bg;
		uint32_t last_color = sc->bg;
		// Repeat the 8th column as the 9th column, except for the
		// Box-drawing characters in Code Page 437.
		if (0xB0 <= sc->uc && sc->uc <= 0xDF && (line_bitmap & 1))
			last_color = sc->fg;
		data[FONT_REALWIDTH] = last_color;
	}
}

// Get the window size.
static void fb_tcgetwinsize(struct screen* base, struct winsize* ws) {
	struct fb* self = container_of(base, struct fb, base);
	ws->ws_col = self->columns;
	ws->ws_row = self->rows;
}

// Clear the console.
static void fb_clear(struct screen* base, const struct screen_char* fill) {
	struct fb* self = container_of(base, struct fb, base);
	memset(self->buffer, 0, self->pitch * self->height);
	for (unsigned short y = 0; y < self->rows; y++) {
		for (unsigned short x = 0; x < self->columns; x++) {
			self->grid[y * MAX_COLUMNS + x] = *fill;
			fb_render(self, x, y);
		}
	}
}

// Set a character on the console.
static void fb_set_char(struct screen* base, const struct screen_char* sc,
                        unsigned short col, unsigned short row) {
	struct fb* self = container_of(base, struct fb, base);
	self->grid[row * MAX_COLUMNS + col] = *sc;
	fb_render(self, col, row);
}

// Scroll the console one line.
static void fb_scroll(struct screen* base, const struct screen_char* fill) {
	struct fb* self = container_of(base, struct fb, base);
	for (unsigned short y = 0; y < self->rows - 1; y++) {
		for (unsigned short x = 0; x < self->columns; x++) {
			size_t to = y * MAX_COLUMNS + x;
			size_t from = (y + 1) * MAX_COLUMNS + x;
			self->grid[to] = self->grid[from];
			fb_render(self, x, y);
		}
	}
	for (unsigned short x = 0; x < self->columns; x++) {
		unsigned short y = self->rows - 1;
		size_t index = y * MAX_COLUMNS + x;
		self->grid[index] = *fill;
		fb_render(self, x, y);
	}
}

const struct screen_ops fb_ops = {
	.tcgetwinsize = fb_tcgetwinsize,
	.clear = fb_clear,
	.set_char = fb_set_char,
	.scroll = fb_scroll,
};
</syntaxhighlight>

==== kernel/kernel/kernel.c ====

<syntaxhighlight lang="c">
#include <stdio.h>

#include <kernel/tty.h>

void kernel_main(void) {
	printf("Hello, \e[31mkernel \e[91;44mWorld\e[m!\n");
}
</syntaxhighlight>

==== kernel/kernel/tty.c ====

<syntaxhighlight lang="c">
#include <kernel/tty.h>

// The /dev/console kernel console.
struct tty* dev_console;

size_t tty_write(struct tty* self, const void* buf, size_t len) {
	for (size_t i = 0; i < len; i++) {
		unsigned char uc = ((unsigned char*) buf)[i];
		if (uc == '\n' /* && (c_oflag & OPOST && c_oflag & ONLCR) */) {
			self->ops->transmit(self, '\r');
			self->ops->transmit(self, '\n');
		} else {
			self->ops->transmit(self, uc);
		}
	}
	return len;
}

size_t console_write(const void* data, size_t size) {
	return tty_write(dev_console, data, size);
}

int console_tcgetwinsize(struct winsize* ws) {
	return dev_console->ops->tcgetwinsize(dev_console, ws);
}
</syntaxhighlight>

==== kernel/kernel/vgafont.h ====

<syntaxhighlight lang="c">
#ifndef KERNEL_VGAFONT_H
#define KERNEL_VGAFONT_H

#include <stdint.h>

static uint8_t vgafont[] = {
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x81, 0xA5, 0x81, 0x81, 0xBD,
	0x99, 0x81, 0x81, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0xFF,
	0xDB, 0xFF, 0xFF, 0xC3, 0xE7, 0xFF, 0xFF, 0x7E, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x6C, 0xFE, 0xFE, 0xFE, 0xFE, 0x7C, 0x38, 0x10,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x38, 0x7C, 0xFE,
	0x7C, 0x38, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18,
	0x3C, 0x3C, 0xE7, 0xE7, 0xE7, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x18, 0x3C, 0x7E, 0xFF, 0xFF, 0x7E, 0x18, 0x18, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x3C,
	0x3C, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
	0xFF, 0xFF, 0xE7, 0xC3, 0xC3, 0xE7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x66, 0x42, 0x42, 0x66, 0x3C, 0x00,
	0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC3, 0x99, 0xBD,
	0xBD, 0x99, 0xC3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x1E, 0x0E,
	0x1A, 0x32, 0x78, 0xCC, 0xCC, 0xCC, 0xCC, 0x78, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x7E, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3F, 0x33, 0x3F, 0x30, 0x30, 0x30,
	0x30, 0x70, 0xF0, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0x63,
	0x7F, 0x63, 0x63, 0x63, 0x63, 0x67, 0xE7, 0xE6, 0xC0, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x18, 0x18, 0xDB, 0x3C, 0xE7, 0x3C, 0xDB, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xC0, 0xE0, 0xF0, 0xF8, 0xFE, 0xF8,
	0xF0, 0xE0, 0xC0, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x06, 0x0E,
	0x1E, 0x3E, 0xFE, 0x3E, 0x1E, 0x0E, 0x06, 0x02, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x18, 0x3C, 0x7E, 0x18, 0x18, 0x18, 0x7E, 0x3C, 0x18, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
	0x66, 0x00, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0xDB,
	0xDB, 0xDB, 0x7B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x7C, 0xC6, 0x60, 0x38, 0x6C, 0xC6, 0xC6, 0x6C, 0x38, 0x0C, 0xC6,
	0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0xFE, 0xFE, 0xFE, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x3C,
	0x7E, 0x18, 0x18, 0x18, 0x7E, 0x3C, 0x18, 0x7E, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x18, 0x3C, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x7E, 0x3C, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x18, 0x0C, 0xFE, 0x0C, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x60, 0xFE, 0x60, 0x30, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0xC0,
	0xC0, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x24, 0x66, 0xFF, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x10, 0x38, 0x38, 0x7C, 0x7C, 0xFE, 0xFE, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xFE, 0x7C, 0x7C,
	0x38, 0x38, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x18, 0x3C, 0x3C, 0x3C, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x66, 0x66, 0x24, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6C,
	0x6C, 0xFE, 0x6C, 0x6C, 0x6C, 0xFE, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00,
	0x18, 0x18, 0x7C, 0xC6, 0xC2, 0xC0, 0x7C, 0x06, 0x06, 0x86, 0xC6, 0x7C,
	0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC2, 0xC6, 0x0C, 0x18,
	0x30, 0x60, 0xC6, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x6C,
	0x6C, 0x38, 0x76, 0xDC, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x30, 0x30, 0x30, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x30,
	0x30, 0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x18,
	0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x7E,
	0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x30, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x02, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x3C, 0x66, 0xC3, 0xC3, 0xDB, 0xDB, 0xC3, 0xC3, 0x66, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x38, 0x78, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6,
	0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0xC6, 0xFE, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x7C, 0xC6, 0x06, 0x06, 0x3C, 0x06, 0x06, 0x06, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x1C, 0x3C, 0x6C, 0xCC, 0xFE,
	0x0C, 0x0C, 0x0C, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xC0,
	0xC0, 0xC0, 0xFC, 0x06, 0x06, 0x06, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x38, 0x60, 0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xC6, 0x06, 0x06, 0x0C, 0x18,
	0x30, 0x30, 0x30, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6,
	0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x06, 0x06, 0x0C, 0x78,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00,
	0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x18, 0x18, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x06,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x00, 0x00,
	0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60,
	0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x7C, 0xC6, 0xC6, 0x0C, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xDE, 0xDE,
	0xDE, 0xDC, 0xC0, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x38,
	0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xFC, 0x66, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x66, 0x66, 0xFC,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x66, 0xC2, 0xC0, 0xC0, 0xC0,
	0xC0, 0xC2, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x6C,
	0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xFE, 0x66, 0x62, 0x68, 0x78, 0x68, 0x60, 0x62, 0x66, 0xFE,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0x66, 0x62, 0x68, 0x78, 0x68,
	0x60, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x66,
	0xC2, 0xC0, 0xC0, 0xDE, 0xC6, 0xC6, 0x66, 0x3A, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1E, 0x0C,
	0x0C, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0xCC, 0x78, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xE6, 0x66, 0x66, 0x6C, 0x78, 0x78, 0x6C, 0x66, 0x66, 0xE6,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x60, 0x60, 0x60, 0x60, 0x60,
	0x60, 0x62, 0x66, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0xE7,
	0xFF, 0xFF, 0xDB, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xC6, 0xE6, 0xF6, 0xFE, 0xDE, 0xCE, 0xC6, 0xC6, 0xC6, 0xC6,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6,
	0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFC, 0x66,
	0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xD6, 0xDE, 0x7C,
	0x0C, 0x0E, 0x00, 0x00, 0x00, 0x00, 0xFC, 0x66, 0x66, 0x66, 0x7C, 0x6C,
	0x66, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6,
	0xC6, 0x60, 0x38, 0x0C, 0x06, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xFF, 0xDB, 0x99, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6,
	0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0xC3,
	0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0x66, 0x3C, 0x18, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xDB, 0xDB, 0xFF, 0x66, 0x66,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0xC3, 0x66, 0x3C, 0x18, 0x18,
	0x3C, 0x66, 0xC3, 0xC3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0xC3,
	0xC3, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xFF, 0xC3, 0x86, 0x0C, 0x18, 0x30, 0x60, 0xC1, 0xC3, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x30, 0x30, 0x30, 0x30, 0x30,
	0x30, 0x30, 0x30, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
	0xC0, 0xE0, 0x70, 0x38, 0x1C, 0x0E, 0x06, 0x02, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00,
	0x30, 0x30, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x0C, 0x7C,
	0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x60,
	0x60, 0x78, 0x6C, 0x66, 0x66, 0x66, 0x66, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC0, 0xC0, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1C, 0x0C, 0x0C, 0x3C, 0x6C, 0xCC,
	0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0xC0, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x38, 0x6C, 0x64, 0x60, 0xF0, 0x60, 0x60, 0x60, 0x60, 0xF0,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76, 0xCC, 0xCC,
	0xCC, 0xCC, 0xCC, 0x7C, 0x0C, 0xCC, 0x78, 0x00, 0x00, 0x00, 0xE0, 0x60,
	0x60, 0x6C, 0x76, 0x66, 0x66, 0x66, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x18, 0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06, 0x00, 0x0E, 0x06, 0x06,
	0x06, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0xE0, 0x60,
	0x60, 0x66, 0x6C, 0x78, 0x78, 0x6C, 0x66, 0xE6, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE6, 0xFF, 0xDB,
	0xDB, 0xDB, 0xDB, 0xDB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xDC, 0x66, 0x66,
	0x66, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x76, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x7C, 0x0C, 0x0C, 0x1E, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0xDC, 0x76, 0x66, 0x60, 0x60, 0x60, 0xF0,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C, 0xC6, 0x60,
	0x38, 0x0C, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x30,
	0x30, 0xFC, 0x30, 0x30, 0x30, 0x30, 0x36, 0x1C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x76,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0xC3, 0xC3,
	0xC3, 0x66, 0x3C, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xC3, 0xC3, 0xC3, 0xDB, 0xDB, 0xFF, 0x66, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0xC3,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0xC6, 0xC6,
	0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0xF8, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xFE, 0xCC, 0x18, 0x30, 0x60, 0xC6, 0xFE, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x0E, 0x18, 0x18, 0x18, 0x70, 0x18, 0x18, 0x18, 0x18, 0x0E,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x70, 0x18,
	0x18, 0x18, 0x0E, 0x18, 0x18, 0x18, 0x18, 0x70, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x38, 0x6C, 0xC6,
	0xC6, 0xC6, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x66,
	0xC2, 0xC0, 0xC0, 0xC0, 0xC2, 0x66, 0x3C, 0x0C, 0x06, 0x7C, 0x00, 0x00,
	0x00, 0x00, 0xCC, 0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x76,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x18, 0x30, 0x00, 0x7C, 0xC6, 0xFE,
	0xC0, 0xC0, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x38, 0x6C,
	0x00, 0x78, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xCC, 0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x30, 0x18, 0x00, 0x78, 0x0C, 0x7C,
	0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x6C, 0x38,
	0x00, 0x78, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x3C, 0x66, 0x60, 0x60, 0x66, 0x3C, 0x0C, 0x06,
	0x3C, 0x00, 0x00, 0x00, 0x00, 0x10, 0x38, 0x6C, 0x00, 0x7C, 0xC6, 0xFE,
	0xC0, 0xC0, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0x00,
	0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0xC0, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x60, 0x30, 0x18, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0xC0, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x00, 0x00, 0x38, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x3C, 0x66,
	0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x60, 0x30, 0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0x00, 0x10, 0x38, 0x6C, 0xC6, 0xC6,
	0xFE, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00, 0x38, 0x6C, 0x38, 0x00,
	0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00,
	0x18, 0x30, 0x60, 0x00, 0xFE, 0x66, 0x60, 0x7C, 0x60, 0x60, 0x66, 0xFE,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6E, 0x3B, 0x1B,
	0x7E, 0xD8, 0xDC, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3E, 0x6C,
	0xCC, 0xCC, 0xFE, 0xCC, 0xCC, 0xCC, 0xCC, 0xCE, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x10, 0x38, 0x6C, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0x00, 0x00, 0x7C, 0xC6, 0xC6,
	0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x30, 0x18,
	0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x30, 0x78, 0xCC, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0x76,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x30, 0x18, 0x00, 0xCC, 0xCC, 0xCC,
	0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0x00,
	0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0x78, 0x00,
	0x00, 0xC6, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6,
	0xC6, 0xC6, 0xC6, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x7E,
	0xC3, 0xC0, 0xC0, 0xC0, 0xC3, 0x7E, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x38, 0x6C, 0x64, 0x60, 0xF0, 0x60, 0x60, 0x60, 0x60, 0xE6, 0xFC,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3, 0x66, 0x3C, 0x18, 0xFF, 0x18,
	0xFF, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFC, 0x66, 0x66,
	0x7C, 0x62, 0x66, 0x6F, 0x66, 0x66, 0x66, 0xF3, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x0E, 0x1B, 0x18, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18,
	0xD8, 0x70, 0x00, 0x00, 0x00, 0x18, 0x30, 0x60, 0x00, 0x78, 0x0C, 0x7C,
	0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x18, 0x30,
	0x00, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x18, 0x30, 0x60, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x30, 0x60, 0x00, 0xCC, 0xCC, 0xCC,
	0xCC, 0xCC, 0xCC, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76, 0xDC,
	0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x00, 0x00, 0x00, 0x00,
	0x76, 0xDC, 0x00, 0xC6, 0xE6, 0xF6, 0xFE, 0xDE, 0xCE, 0xC6, 0xC6, 0xC6,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x6C, 0x6C, 0x3E, 0x00, 0x7E, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x6C, 0x6C,
	0x38, 0x00, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x30, 0x30, 0x00, 0x30, 0x30, 0x60, 0xC0, 0xC6, 0xC6, 0x7C,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xC0,
	0xC0, 0xC0, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0xFE, 0x06, 0x06, 0x06, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xC0, 0xC0, 0xC2, 0xC6, 0xCC, 0x18, 0x30, 0x60, 0xCE, 0x9B, 0x06,
	0x0C, 0x1F, 0x00, 0x00, 0x00, 0xC0, 0xC0, 0xC2, 0xC6, 0xCC, 0x18, 0x30,
	0x66, 0xCE, 0x96, 0x3E, 0x06, 0x06, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18,
	0x00, 0x18, 0x18, 0x18, 0x3C, 0x3C, 0x3C, 0x18, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x6C, 0xD8, 0x6C, 0x36, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD8, 0x6C, 0x36,
	0x6C, 0xD8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x44, 0x11, 0x44,
	0x11, 0x44, 0x11, 0x44, 0x11, 0x44, 0x11, 0x44, 0x11, 0x44, 0x11, 0x44,
	0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA,
	0x55, 0xAA, 0x55, 0xAA, 0xDD, 0x77, 0xDD, 0x77, 0xDD, 0x77, 0xDD, 0x77,
	0xDD, 0x77, 0xDD, 0x77, 0xDD, 0x77, 0xDD, 0x77, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0xF8, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0xF8, 0x18, 0xF8,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0xF6, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x18, 0xF8,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x36, 0x36, 0x36, 0x36,
	0x36, 0xF6, 0x06, 0xF6, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0x06, 0xF6,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x36, 0xF6, 0x06, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0xFE, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0xF8, 0x18, 0xF8,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0xF8, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1F, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0xFF, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1F, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0xFF, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x1F, 0x18, 0x1F, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x37,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x37, 0x30, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x3F, 0x30, 0x37, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0xF7, 0x00, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xFF, 0x00, 0xF7, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x37, 0x30, 0x37, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x36, 0x36, 0x36,
	0x36, 0xF7, 0x00, 0xF7, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x18, 0x18, 0x18, 0x18, 0x18, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0xFF,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xFF, 0x00, 0xFF, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x3F,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x1F, 0x18, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x1F, 0x18, 0x1F, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3F,
	0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x36, 0x36, 0x36, 0xFF, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36, 0x36,
	0x18, 0x18, 0x18, 0x18, 0x18, 0xFF, 0x18, 0xFF, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0xF8,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x1F, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
	0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
	0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xF0, 0xF0, 0xF0, 0xF0,
	0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0,
	0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
	0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x76, 0xDC, 0xD8, 0xD8, 0xD8, 0xDC, 0x76, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x78, 0xCC, 0xCC, 0xCC, 0xD8, 0xCC, 0xC6, 0xC6, 0xC6, 0xCC,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xC6, 0xC6, 0xC0, 0xC0, 0xC0,
	0xC0, 0xC0, 0xC0, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0xFE, 0x6C, 0x6C, 0x6C, 0x6C, 0x6C, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0xFE, 0xC6, 0x60, 0x30, 0x18, 0x30, 0x60, 0xC6, 0xFE,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0xD8, 0xD8,
	0xD8, 0xD8, 0xD8, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x66, 0x66, 0x66, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xC0, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x76, 0xDC, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x18, 0x3C, 0x66, 0x66,
	0x66, 0x3C, 0x18, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38,
	0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0x6C, 0x38, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x38, 0x6C, 0xC6, 0xC6, 0xC6, 0x6C, 0x6C, 0x6C, 0x6C, 0xEE,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1E, 0x30, 0x18, 0x0C, 0x3E, 0x66,
	0x66, 0x66, 0x66, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x7E, 0xDB, 0xDB, 0xDB, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x03, 0x06, 0x7E, 0xDB, 0xDB, 0xF3, 0x7E, 0x60, 0xC0,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1C, 0x30, 0x60, 0x60, 0x7C, 0x60,
	0x60, 0x60, 0x30, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7C,
	0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0xFE, 0x00, 0x00, 0xFE, 0x00, 0x00, 0xFE, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x7E, 0x18,
	0x18, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30,
	0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00, 0x7E,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0E, 0x1B, 0x1B, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	0x18, 0x18, 0x18, 0x18, 0xD8, 0xD8, 0xD8, 0x70, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x7E, 0x00, 0x18, 0x18, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x76, 0xDC, 0x00,
	0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x6C, 0x6C,
	0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0C, 0x0C,
	0x0C, 0x0C, 0x0C, 0xEC, 0x6C, 0x6C, 0x3C, 0x1C, 0x00, 0x00, 0x00, 0x00,
	0x00, 0xD8, 0x6C, 0x6C, 0x6C, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x70, 0xD8, 0x30, 0x60, 0xC8, 0xF8, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x7C, 0x7C, 0x7C, 0x7C, 0x7C, 0x7C, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x00, 0x00, 0x00,
};

#endif
</syntaxhighlight>

==== kernel/kernel/vt.c ====

<syntaxhighlight lang="c">
#include <string.h>

#include <kernel/kernel.h>
#include <kernel/screen.h>
#include <kernel/tty.h>
#include <kernel/vga.h>
#include <kernel/vt.h>

// RGB palette for the first 16 ANSI colors.
static const uint32_t palette[16] = {
	0x000000, 0xcc0000, 0x3e9a06, 0xc4a000,
	0x3465a4, 0x75507b, 0x06989a, 0xbfbfbf,
	0x555753, 0xef2929, 0x8ae234, 0xfce94f,
	0x729fcf, 0xad7fa8, 0x34e2e2, 0xffffff,
};

// VGA colors for the first 8 ANSI colors. The high bit is used for bright.
static const uint8_t ansi_to_vga_color[8] = {
	VGA_COLOR_BLACK, VGA_COLOR_RED, VGA_COLOR_GREEN, VGA_COLOR_BROWN,
	VGA_COLOR_BLUE, VGA_COLOR_MAGENTA, VGA_COLOR_CYAN, VGA_COLOR_LIGHT_GREY,
};

static const uint8_t DEFAULT_VGACOLOR =
	VGA_COLOR_LIGHT_GREY | VGA_COLOR_BLACK << 4;
static const unsigned int DEFAULT_FOREGROUND = 7;
static const unsigned int DEFAULT_BACKGROUND = 0;

// Initialize a vt (virtual terminal) object connected to a screen driver.
struct tty* vt_init(struct vt* self, struct screen* screen) {
	memset(self, 0, sizeof(*self));
	self->base.ops = &vt_ops;
	self->screen = screen;
	self->row = 0;
	self->column = 0;
	self->vgacolor = DEFAULT_VGACOLOR;
	self->fgcolor = palette[DEFAULT_FOREGROUND];
	self->bgcolor = palette[DEFAULT_BACKGROUND];
	struct winsize ws;
	self->screen->ops->tcgetwinsize(self->screen, &ws);
	self->width = ws.ws_col;
	self->height = ws.ws_row;
	struct screen_char fill = {
		.uc = ' ', .vgacolor = self->vgacolor, .attr = self->attr,
		.fg = self->fgcolor, .bg = self->bgcolor
	};
	self->screen->ops->clear(self->screen, &fill);
	return &self->base;
}

// Move to the next line and scroll if needed.
static void vt_next_line(struct vt* self) {
	if (self->height <= self->row + 1) {
		struct screen_char fill = {
			.uc = ' ', .vgacolor = self->vgacolor, .attr = self->attr,
			.fg = self->fgcolor, .bg = self->bgcolor
		};
		self->screen->ops->scroll(self->screen, &fill);
	} else
		self->row++;
}

// Execute an ANSI escape sequence command.
static void vt_run_ansi(struct vt* self, char c) {
	switch (c) {
	case 'm':
		if (self->ansi_count == 0) {
			// No parameters means turn all attributes off.
			self->ansi_params[0] = 0;
			self->ansi_count = 1;
		}
		// Execute each of the parameters to the command.
		for (size_t i = 0; i < self->ansi_count; i++) {
			unsigned int cmd = self->ansi_params[i];
			if (cmd == 0) {
				// Turn all attributes off.
				self->vgacolor = DEFAULT_VGACOLOR;
				self->attr = 0;
				self->fgcolor = palette[DEFAULT_FOREGROUND];
				self->bgcolor = palette[DEFAULT_BACKGROUND];
			} else if (30 <= cmd && cmd <= 37) {
				// Set text color.
				unsigned int value = cmd - 30;
				self->vgacolor &= 0xF0;
				self->vgacolor |= ansi_to_vga_color[value] << 0;
				self->fgcolor = palette[value];
			} else if (cmd == 39) {
				// Set default text color.
				self->vgacolor &= 0xF0;
				self->vgacolor |= DEFAULT_VGACOLOR & 0x0F;
				self->fgcolor = palette[DEFAULT_FOREGROUND];
			} else if (40 <= cmd && cmd <= 47) {
				// Set background color.
				unsigned int value = cmd - 40;
				self->vgacolor &= 0x0F;
				self->vgacolor |= ansi_to_vga_color[value] << 4;
				self->bgcolor = palette[value];
			} else if (cmd == 49) {
				// Set default background color.
				self->vgacolor &= 0x0F;
				self->vgacolor |= DEFAULT_VGACOLOR & 0xF0;
				self->bgcolor = palette[DEFAULT_BACKGROUND];
			} else if (90 <= cmd && cmd <= 97) {
				// Set text color.
				unsigned int value = cmd - 90 + 8;
				self->vgacolor &= 0xF0;
				self->vgacolor |= (0x8 | ansi_to_vga_color[value - 8]) << 0;
				self->fgcolor = palette[value];
			} else if (100 <= cmd && cmd <= 107) {
				// Set background color.
				unsigned int value = cmd - 100 + 8;
				self->vgacolor &= 0x0F;
				self->vgacolor |= (0x8 | ansi_to_vga_color[value - 8]) << 4;
				self->bgcolor = palette[value];
			} else {
				self->ansi_state = ANSI_STATE_NONE;
			}
		}
		break;
	}
}

// Process the next byte in an ANSI escape sequence.
static void vt_escaped(struct vt* self, char c) {
	if (self->ansi_state == ANSI_STATE_CSI) {
		// Begin a \e[ terminal escape sequence.
		if (c == '[')
			self->ansi_state = ANSI_STATE_COMMAND;
		// Return to normal processing on any unrecognized escape sequence.
		else
			self->ansi_state = ANSI_STATE_NONE;
		return;
	} else if (self->ansi_state == ANSI_STATE_COMMAND) {
		if ('0' <= c && c <= '9') {
			// Parse numeric parameters to the escape sequence.
			if (self->ansi_count == 0)
				self->ansi_count = 1;
			if (self->ansi_count <= ANSI_PARAM_MAX) {
				self->ansi_params[self->ansi_count - 1] *= 10;
				self->ansi_params[self->ansi_count - 1] += c - '0';
			}
		} else if (c == ';') {
			// Parse another parameter.
			if (self->ansi_count < ANSI_PARAM_MAX)
				self->ansi_count++;
			else
				self->ansi_state = ANSI_STATE_NONE;
		} else if ('@' <= c && c <= '~') {
			// Execute the command if the byte is in the right range.
			vt_run_ansi(self, c);
			self->ansi_state = ANSI_STATE_NONE;
		} else {
			// Otherwise ignore any unrecognized escape sequence.
			self->ansi_state = ANSI_STATE_NONE;
		}
	 }
}

// Process the next output byte.
static bool vt_transmit(struct tty* base, unsigned char uc) {
	struct vt* self = container_of(base, struct vt, base);
	if (self->ansi_state) {
		// Process any ongoing terminal escape sequences.
		vt_escaped(self, uc);
	} else if (uc == '\e') {
		// Begin a terminal escape sequence.
		self->ansi_state = ANSI_STATE_CSI;
		self->ansi_count = 0;
		memset(self->ansi_params, 0, sizeof(self->ansi_params));
	} else if (uc == '\n') {
		// Handle a newline without a carriage return.
		vt_next_line(self);
	} else if (uc == '\r') {
		// Handle a carriage return.
		self->column = 0;
	} else {
		// Otherwise render the character.
		// The cursor is allowed to be at the edge of the row if it is full, in
		// that case move to the new line when the next character appears.
		if (self->width <= self->column) {
			self->column = 0;
			vt_next_line(self);
		}
		struct screen_char sc = {
			.uc = uc, .vgacolor = self->vgacolor, .attr = self->attr,
			.fg = self->fgcolor, .bg = self->bgcolor
		};
		self->screen->ops->set_char(self->screen, &sc, self->column, self->row);
		self->column++;
	}
	return true;
}

// Get the terminal window size.
static int vt_tcgetwinsize(struct tty* base, struct winsize* ws) {
	struct vt* self = container_of(base, struct vt, base);
	self->screen->ops->tcgetwinsize(self->screen, ws);
	return 0;
}

const struct tty_ops vt_ops = {
	.transmit = vt_transmit,
	.tcgetwinsize = vt_tcgetwinsize,
};
</syntaxhighlight>

=== libc and libk ===

These files go into the libc directory.

==== libc/.gitignore ====

<syntaxhighlight>
*.a
*.d
*.o
</syntaxhighlight>

==== libc/Makefile ====

<syntaxhighlight lang="make">
# To cross-compile MyOS: make HOST=i386-elf SYSROOT=../sysroot
# By default the build system will assume it's running on MyOS.

include ../build-aux/common.mk

CFLAGS?=-O2 -g
CPPFLAGS?=
LDFLAGS?=
LIBS?=

CFLAGS:=$(CFLAGS) -ffreestanding -Wall -Wextra
CPPFLAGS:=$(CPPFLAGS) -D__is_libc -Iinclude
LIBK_CFLAGS:=$(CFLAGS)
LIBK_CPPFLAGS:=$(CPPFLAGS) -D__is_libk

ARCHDIR=arch/$(HOSTARCH)

-include $(ARCHDIR)/arch.mk

CFLAGS:=$(CFLAGS) $(ARCH_CFLAGS)
CPPFLAGS:=$(CPPFLAGS) $(ARCH_CPPFLAGS)
LIBK_CFLAGS:=$(LIBK_CFLAGS) $(KERNEL_ARCH_CFLAGS)
LIBK_CPPFLAGS:=$(LIBK_CPPFLAGS) $(KERNEL_ARCH_CPPFLAGS)

FREEOBJS=\
$(ARCH_FREEOBJS) \
stdio/printf.o \
stdio/putchar.o \
stdio/puts.o \
stdlib/abort.o \
string/memcmp.o \
string/memcpy.o \
string/memmove.o \
string/memset.o \
string/strcmp.o \
string/strlen.o \

HOSTEDOBJS=\
$(ARCH_HOSTEDOBJS) \

OBJS=\
$(FREEOBJS) \
$(HOSTEDOBJS) \

LIBK_OBJS=$(FREEOBJS:.o=.libk.o)

#BINARIES=libc.a libk.a # Not ready for libc yet.
BINARIES=libk.a

.PHONY: all clean distclean install install-headers install-libs
.SUFFIXES: .o .libk.o .c .S

all: $(BINARIES)

libc.a: $(OBJS)
	$(AR) rcs $@ $(OBJS)

libk.a: $(LIBK_OBJS)
	$(AR) rcs $@ $(LIBK_OBJS)

.c.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(CFLAGS) $(CPPFLAGS)

.S.o:
	$(CC) -MD -c $< -o $@ $(CFLAGS) $(CPPFLAGS)

.c.libk.o:
	$(CC) -MD -c $< -o $@ -std=gnu11 $(LIBK_CFLAGS) $(LIBK_CPPFLAGS)

.S.libk.o:
	$(CC) -MD -c $< -o $@ $(LIBK_CFLAGS) $(LIBK_CPPFLAGS)

clean:
	rm -f $(BINARIES)
	find . -name "*.a" -delete
	find . -name "*.d" -delete
	find . -name "*.o" -delete

distclean: clean

install: install-headers install-libs

install-headers:
	mkdir -p $(DESTDIR)$(INCLUDEDIR)
	cp -R --preserve=timestamps include/. $(DESTDIR)$(INCLUDEDIR)/.

install-libs: $(BINARIES)
	mkdir -p $(DESTDIR)$(LIBDIR)
	cp $(BINARIES) $(DESTDIR)$(LIBDIR)

-include $(OBJS:.o=.d)
-include $(LIBK_OBJS:.o=.d)
</syntaxhighlight>

==== libc/arch/i386/arch.mk ====

<syntaxhighlight lang="make">
ARCH_CFLAGS=
ARCH_CPPFLAGS=
KERNEL_ARCH_CFLAGS=
KERNEL_ARCH_CPPFLAGS=

ARCH_FREEOBJS=\

ARCH_HOSTEDOBJS=\
</syntaxhighlight>

==== libc/include/stdio.h ====

<syntaxhighlight lang="c">
#ifndef _STDIO_H
#define _STDIO_H 1

#include <sys/cdefs.h>

#define EOF (-1)

#ifdef __cplusplus
extern "C" {
#endif

int printf(const char* __restrict, ...);
int putchar(int);
int puts(const char*);

#ifdef __cplusplus
}
#endif

#endif
</syntaxhighlight>

==== libc/include/stdlib.h ====

<syntaxhighlight lang="c">
#ifndef _STDLIB_H
#define _STDLIB_H 1

#include <sys/cdefs.h>

#ifdef __cplusplus
extern "C" {
#endif

__attribute__((__noreturn__))
void abort(void);

#ifdef __cplusplus
}
#endif

#endif
</syntaxhighlight>

==== libc/include/string.h ====

<syntaxhighlight lang="c">
#ifndef _STRING_H
#define _STRING_H 1

#include <sys/cdefs.h>

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int memcmp(const void*, const void*, size_t);
void* memcpy(void* __restrict, const void* __restrict, size_t);
void* memmove(void*, const void*, size_t);
void* memset(void*, int, size_t);
int strcmp(const char*, const char*);
size_t strlen(const char*);

#ifdef __cplusplus
}
#endif

#endif
</syntaxhighlight>

==== libc/include/sys/cdefs.h ====

<syntaxhighlight lang="c">
#ifndef _SYS_CDEFS_H
#define _SYS_CDEFS_H 1

#define __myos_libc 1

#endif
</syntaxhighlight>

==== libc/stdio/printf.c ====

<syntaxhighlight lang="c">
#include <limits.h>
#include <stdbool.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static bool print(const char* data, size_t length) {
	const unsigned char* bytes = (const unsigned char*) data;
	for (size_t i = 0; i < length; i++)
		if (putchar(bytes[i]) == EOF)
			return false;
	return true;
}

int printf(const char* restrict format, ...) {
	va_list parameters;
	va_start(parameters, format);

	int written = 0;

	while (*format != '\0') {
		size_t maxrem = INT_MAX - written;

		if (format[0] != '%' || format[1] == '%') {
			if (format[0] == '%')
				format++;
			size_t amount = 1;
			while (format[amount] && format[amount] != '%')
				amount++;
			if (maxrem < amount) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(format, amount))
				return -1;
			format += amount;
			written += amount;
			continue;
		}

		const char* format_begun_at = format++;

		if (*format == 'c') {
			format++;
			char c = (char) va_arg(parameters, int /* char promotes to int */);
			if (!maxrem) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(&c, sizeof(c)))
				return -1;
			written++;
		} else if (*format == 's') {
			format++;
			const char* str = va_arg(parameters, const char*);
			size_t len = strlen(str);
			if (maxrem < len) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(str, len))
				return -1;
			written += len;
		} else {
			format = format_begun_at;
			size_t len = strlen(format);
			if (maxrem < len) {
				// TODO: Set errno to EOVERFLOW.
				return -1;
			}
			if (!print(format, len))
				return -1;
			written += len;
			format += len;
		}
	}

	va_end(parameters);
	return written;
}
</syntaxhighlight>

==== libc/stdio/putchar.c ====

<syntaxhighlight lang="c">
#include <stdio.h>

#if defined(__is_libk)
#include <kernel/tty.h>
#endif

int putchar(int ic) {
#if defined(__is_libk)
	char c = (char) ic;
	console_write(&c, sizeof(c));
#else
	// TODO: Implement stdio and the write system call.
#endif
	return ic;
}
</syntaxhighlight>

==== libc/stdio/puts.c ====

<syntaxhighlight lang="c">
#include <stdio.h>

int puts(const char* string) {
	return printf("%s\n", string);
}
</syntaxhighlight>

==== libc/stdlib/abort.c ====

<syntaxhighlight lang="c">
#include <stdio.h>
#include <stdlib.h>

__attribute__((__noreturn__))
void abort(void) {
#if defined(__is_libk)
	// TODO: Add proper kernel panic.
	printf("kernel: panic: abort()\n");
#else
	// TODO: Abnormally terminate the process as if by SIGABRT.
	printf("abort()\n");
#endif
	while (1) {
#if defined(__i386__)
		asm volatile ("hlt");
#endif
	}
	__builtin_unreachable();
}
</syntaxhighlight>

==== libc/string/memcmp.c ====

<syntaxhighlight lang="c">
#include <string.h>

int memcmp(const void* aptr, const void* bptr, size_t size) {
	const unsigned char* a = (const unsigned char*) aptr;
	const unsigned char* b = (const unsigned char*) bptr;
	for (size_t i = 0; i < size; i++) {
		if (a[i] < b[i])
			return -1;
		else if (b[i] < a[i])
			return 1;
	}
	return 0;
}
</syntaxhighlight>

==== libc/string/memcpy.c ====

<syntaxhighlight lang="c">
#include <string.h>

void* memcpy(void* restrict dstptr, const void* restrict srcptr, size_t size) {
	unsigned char* dst = (unsigned char*) dstptr;
	const unsigned char* src = (const unsigned char*) srcptr;
	for (size_t i = 0; i < size; i++)
		dst[i] = src[i];
	return dstptr;
}
</syntaxhighlight>

==== libc/string/memmove.c ====

<syntaxhighlight lang="c">
#include <string.h>

void* memmove(void* dstptr, const void* srcptr, size_t size) {
	unsigned char* dst = (unsigned char*) dstptr;
	const unsigned char* src = (const unsigned char*) srcptr;
	if (dst < src) {
		for (size_t i = 0; i < size; i++)
			dst[i] = src[i];
	} else {
		for (size_t i = size; i != 0; i--)
			dst[i-1] = src[i-1];
	}
	return dstptr;
}
</syntaxhighlight>

==== libc/string/memset.c ====

<syntaxhighlight lang="c">
#include <string.h>

void* memset(void* bufptr, int value, size_t size) {
	unsigned char* buf = (unsigned char*) bufptr;
	for (size_t i = 0; i < size; i++)
		buf[i] = (unsigned char) value;
	return bufptr;
}
</syntaxhighlight>

==== libc/string/strcmp.c ====

<syntaxhighlight lang="c">
#include <stdbool.h>
#include <string.h>

int strcmp(const char* aptr, const char* bptr) {
	const unsigned char* a = (const unsigned char*) aptr;
	const unsigned char* b = (const unsigned char*) bptr;
	for (size_t i = 0; true; i++) {
		if (a[i] == '\0' && b[i] == '\0')
			return 0;
		if (a[i] < b[i])
			return -1;
		if (a[i] > b[i])
			return 1;
	}
}
</syntaxhighlight>

==== libc/string/strlen.c ====

<syntaxhighlight lang="c">
#include <string.h>

size_t strlen(const char* str) {
	size_t len = 0;
	while (str[len])
		len++;
	return len;
}
</syntaxhighlight>

=== build-aux ===

These files go into the build-aux directory.

==== build-aux/common.mk ====

<syntaxhighlight lang="make">
# This build system handles cross-compilation, where the compilation happens on
# the machine BUILD, and produces executables that will run on machine HOST,
# whose output (if any) would run on machine TARGET. By default, it will assume
# BUILD == HOST == TARGET, unless overridden.

# Determine the platform per uname.
UNAME_OS:=$(shell uname -s | tr '[:upper:]' '[:lower:]' | sed 's,^linux$$,linux-gnu,g')
UNAME_MACHINE:=$(shell uname -m)

# Detect the platform that the software is being built on.
BUILD?=$(UNAME_MACHINE)-$(UNAME_OS)

# Determine the platform the software will run on.
HOST?=$(BUILD)
HOST_MACHINE:=$(shell expr x$(HOST) : 'x\([^-]*\).*')

# Determine the platform the software will target.
TARGET?=$(HOST)

# Determine the per-arch directory. For i*86-elf, use i386.
HOSTARCH:=$(if $(filter i%86,$(HOST_MACHINE)),i386,$(HOST_MACHINE))

# Determine the prefix for host tools.
ifneq ($(BUILD),$(HOST))
  HOST_TOOL_PREFIX?=$(HOST)-
endif

# Utility function to determine whether a variable is unset or its value is a
# default provided by make. Those defaults must be overridden below, but any
# other user choices must be respected.
is_unset_or_default = $(filter undefined default,$(origin $(1)))

# Determine the names of the cross-tools that target the host platform.
ifneq ($(BUILD),$(HOST))
  ifneq ($(call is_unset_or_default,CC),)
    CC:=$(HOST_TOOL_PREFIX)gcc
  endif
  ifneq ($(call is_unset_or_default,AR),)
    AR:=$(HOST_TOOL_PREFIX)ar
  endif
  ifneq ($(call is_unset_or_default,AS),)
    AS:=$(HOST_TOOL_PREFIX)as
  endif
endif

# Inform the toolchain about the location of the sysroot if overridden.
ifdef SYSROOT
  CC:=$(CC) --sysroot="$(SYSROOT)"
endif

# Determine the system directory structure.
PREFIX?=/usr
EXEC_PREFIX?=$(PREFIX)
BOOTDIR?=/boot
LIBDIR?=$(EXEC_PREFIX)/lib
INCLUDEDIR?=$(PREFIX)/include

# Work around that the -elf gcc targets doesn't have a system include directory
# because it was configured with --without-headers rather than --with-sysroot.
ifneq ($(filter %-elf,$(HOST)),)
  CC+=-isystem=$(INCLUDEDIR)
endif
</syntaxhighlight>

==== build-aux/grub.cfg ====

<syntaxhighlight>
menuentry "myos" {
	multiboot2 /boot/myos
}
menuentry "myos (serial)" {
	multiboot2 /boot/myos --console=ttyS0
}
</syntaxhighlight>

=== Miscellaneous ===

These files go into the root source directory.

==== .gitignore ====

<syntaxhighlight>
*.iso
isodir
sysroot
</syntaxhighlight>

==== Makefile ====

<syntaxhighlight lang="make">
# To cross-compile MyOS: make HOST=i386-elf
# By default the build system will assume it's running on MyOS.

include build-aux/common.mk

ifndef SYSROOT
  SYSROOT:=$(shell pwd)/sysroot
endif

MODULES=\
libc \
kernel \

QEMU?=qemu-system-$(HOSTARCH)
QEMU_ENABLE_KVM?=-enable-kvm
QEMU_OPTIONS?=

.PHONY: all
all: sysroot

.PHONY: sysroot-headers
sysroot-headers:
	export SYSROOT="$(SYSROOT)" && export DESTDIR="$(SYSROOT)" && \
	(for D in libc kernel; do $(MAKE) -C $$D install-headers || exit $$?; done)

.PHONY: sysroot
sysroot: sysroot-headers
	export SYSROOT="$(SYSROOT)" && export DESTDIR="$(SYSROOT)" && \
	(for D in $(MODULES); do $(MAKE) -C $$D install || exit $$?; done)

.PHONY: clean
clean:
	export SYSROOT="$(SYSROOT)" && export DESTDIR="$(SYSROOT)" && \
	(for D in $(MODULES); do $(MAKE) -C $$D clean || exit $$?; done)

.PHONY: distclean
distclean: clean
	rm -rf "$(SYSROOT)"
	rm -rf isodir
	rm -f myos.iso

isodir/boot/grub/grub.cfg: build-aux/grub.cfg
	mkdir -p isodir/boot/grub
	cp build-aux/grub.cfg $@

isodir/boot/myos: sysroot
	mkdir -p isodir/boot
	cp "$(SYSROOT)$(BOOTDIR)/myos" $@

myos.iso: isodir/boot/grub/grub.cfg isodir/boot/myos
	grub-mkrescue -o myos.iso isodir

.PHONY: iso
iso: myos.iso

.PHONY: qemu
qemu: myos.iso
	$(QEMU) $(QEMU_ENABLE_KVM) $(QEMU_OPTIONS) -cdrom myos.iso
</syntaxhighlight>

== Building ==

To build the system:

<syntaxhighlight lang="bash">
make HOST=i686-elf
</syntaxhighlight>

Or to just build one module:

<syntaxhighlight lang="bash">
cd libc && make HOST=i686-elf SYSROOT=../sysroot
</syntaxhighlight>

To build the system and make a bootable CD-ROM image:

<syntaxhighlight lang="bash">
make HOST=i686-elf myos.iso
</syntaxhighlight>

To build the system and make a bootable CD-ROM image and launch it in [[Qemu]]:

<syntaxhighlight lang="bash">
make HOST=i686-elf qemu
</syntaxhighlight>

'''Note:''' If qemu doesn't have hardware acceleration on your platform, add an
empty assignment <tt>QEMU_ENABLE_KVM=</tt> to the make invocation.

To use use the ttyS serial line driver, redirect the Qemu serial line to
somewhere that you can use it, and select the <tt>myos (serial)</tt> bootloader
menu option:

<syntaxhighlight lang="bash">
make HOST=i686-elf QEMU_OPTIONS='-serial stdio' qemu
</syntaxhighlight>

== Troubleshooting ==

=== Double check the source code ===

If you receive odd errors during the build, you may have made a mistake during manual copying, perhaps missed a file, forgot to make a file executable, or bugs in the highlighting software we use cause unintended whitespace to appear. Perform a git repository clone as described above, and use that code instead, or compare the two directory trees with the <tt>diff(1)</tt> diff command line utility. If you made personal changes to the code, those may be at fault.

=== Forgetting to set HOST ===

<syntaxhighlight>
/usr/lib/gcc/x86_64-linux-gnu/15/include/limits.h:210:15: fatal error: limits.h: No such file or directory
  210 | #include_next <limits.h>                /* recurse down to the real one */
</syntaxhighlight>

The solution is to properly cross-compile as shown above. This error will occur
if the <tt>HOST=i686-elf</tt> environment variable is not set, which causes the
build system to not cross-compile and instead invoke the system compiler
directly. In this case, the build may partially succeed, but the compiler's
<tt>limits.h</tt> header wrapper tries to include the real <tt>limits.h</tt>
header from libc, but no such header exists in this example.

During the [[GCC]] build, it remembers whether a libc <tt>limits.h</tt> header
exists, and wraps the libc header using <tt>#include_next</tt> if it does, and
otherwise provides its own header. The <tt>i686-elf</tt> toolchain is built
using <tt>--without-headers</tt> and it includes its own <tt>limits.h</tt>.
You can provide your own <tt>limits.h</tt> header only when completing the
[[OS Specific Toolchain]] tutorial in the future. 

== Moving Forward ==

Please note that this tutorial is an example. You are personally responsible for
adopting the code to your needs and you need to fully understand everything. The
example here is complex, but the complexity is very intentional, and is actually
designed to make your design much simpler in the long run. There are a lot of
subtleties to study, much of which is explained above, and other aspects that
can be learned from the code. You are responsible for becoming an expert in the
topics introduced in this tutorial. Welcome to osdev!

You should adapt this template to your needs. There's a number of things you
should consider doing now:

=== Renaming MyOS to YourOS ===

Certainly you wish to name your operating system after your favorite flower,
hometown, boolean value, or whatever marketing told you. Do a search and replace
that replaces <tt>myos</tt> with whatever you wish to call it. Keep in mind that
the name is deliberately lower-case in a few places for technical reasons.

=== Serial Line ===

The [[GRUB]] bootloader can be configured to use the serial line too, for a
fully headless experience:

<syntaxhighlight lang="bash">
serial
terminal_input serial
terminal_output serial
menuentry "myos (serial)" {
	multiboot2 /boot/myos --console=ttyS0
}
</syntaxhighlight>

Ideally the top-level makefile would invoke a script to dynamically generate the
<tt>isodir/boot/grub/grub.cfg</tt> GRUB configuration file. This feature would
have an option to enable the serial line by default. Designing this feature is
left as an exercise.

=== Improving the Build System ===
{{Main|Hard Build System}}

It is probably worth improving the build system.

It's worth considering how contributors will build your operating system. It's an easy trap to fall into thinking you can make a super script that does everything. This design will end up complex and insufficiently flexible; or it will be flexible and even more complex. It's better to document what the user should do to prepare a cross toolchain and what prerequisite programs to install. This tutorial shows an example hard build system that merely builds the operating system. You can complete it by documenting how to build a cross-compiler and how to use it.

=== Stack Smash Protector ===
{{Main|Stack Smashing Protector}}

Early is not too soon to think about security and robustness. You can take advantage of the optional stack smash protector offered by modern compilers that detect stack buffer overruns rather than behaving unexpectedly (or nothing happening, if unlucky).

=== Going Further ===
{{Main|Going Further on x86}}
This guide is meant as an overview of what to do, so you have a kernel ready for more features, without actually redesigning it radically when adding them.

=== User-Space ===

A later tutorial in this series will extend this template with a proper
user-space and an [[OS Specific Toolchain]] that fully utilizes the system root.

[[Category:Tutorials]]
[[Category:C]]
````

<a id="w003637"></a>

## W003637 — Hosted GCC Cross-Compiler

Bron-ID: W003637
Namespace: Main (0)
Revisie: 30283 | Datum: 2026-03-16T18:03:27Z
Bron: https://wiki.osdev.org/Hosted_GCC_Cross-Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=Hosted_GCC_Cross-Compiler&oldid=30283

````text
{{rating|3}}
In this tutorial we create a cross-compiler that provides a hosted environment (user-space, libc, all that good stuff) as opposed to the initial [[GCC Cross-Compiler]] that only provides a freestanding environment. This is an advanced topic meant for established developers, newcomers should start with a [[GCC Cross-Compiler|freestanding cross-compiler]]. The <tt>-elf</tt> targets are incapable of having a user-space, so you need to make a [[OS Specific Toolchain]] before following this tutorial. You also need a [[C Library]], perhaps a [[Creating a C Library|homemade one]].

== Introduction ==

This tutorial acts as a patch to the original [[GCC Cross-Compiler]] article by subtly changing the instructions. The caveats and instructions from it generally apply here, too. You are assumed to be an experienced reader that automatically remembers trivial matters such as adding the cross-compiler prefix bindir to the <tt>PATH</tt> before building the cross-compiler, or installing the dependencies of Binutils and GCC.

This article completes the work begun in [[OS Specific Toolchain]] by compiling it, thus implementing the next step in [[Porting GCC to your OS]]: Teaching GCC what your operating system is and making a cross-compiler for it.

You will be using a new [[Target Triplet|target triplet]], rather than <tt>i686-elf</tt>, we'll be using your custom <tt>i686-myos</tt> target. You will be supplying the Binutils and GCC source code yourself rather than using pure upstream releases.

== Prerequisites ==

To correctly complete this tutorial, your libc must meet particular minimum requirements. The libgcc will be built with libc support, meaning that your libc needs a few standard headers that define particular types, constants and functions. In particular, you need to supply:

* <tt>sys/types.h</tt>: This header is just required to exist and can be empty.
* <tt>errno.h</tt>: This header is just required to exist and can be empty.
* <tt>stdlib.h</tt>: <tt>abort()</tt>, <tt>free()</tt>, <tt>malloc()</tt>.
* <tt>stdio.h</tt>: <tt>FILE</tt>, <tt>stderr</tt>, <tt>fflush()</tt>, <tt>fprintf()</tt>.
* <tt>string.h</tt>: <tt>size_t</tt>, <tt>memcpy</tt>, <tt>memset()</tt>, <tt>strlen()</tt>.
* <tt>time.h</tt>: This header is just required to exist and can be empty.
* <tt>unistd.h</tt>: This header is just required to exist and can be empty.

Additionally, the <tt>all-target-libgcc</tt> target also unconditionally builds the libgcov support library, which has these additional requirements:

* <tt>sys/types.h</tt>: <tt>pid_t</tt>.
* <tt>stdlib.h</tt>: <tt>atexit()</tt>, <tt>atoi()</tt>, <tt>getenv()</tt>, <tt>malloc()</tt>, <tt>calloc()</tt>, <tt>abs()</tt>.
* <tt>stdio.h</tt>: <tt>size_t</tt>, <tt>SEEK_SET</tt>, <tt>fclose()</tt>, <tt>fopen()</tt>, <tt>fread()</tt>, <tt>fseek()</tt>, <tt>ftell()</tt>, <tt>fwrite()</tt>, <tt>setbuf()</tt>, <tt>vfprintf()</tt>, <tt>sprintf()</tt>.
* <tt>string.h</tt>: <tt>strcpy()</tt>, <tt>strcat()</tt>, <tt>strchr()</tt>.
* <tt>unistd.h</tt>: <tt>pid_t</tt>, <tt>intptr_t</tt>, <tt>fork()</tt>, <tt>execv()</tt>, <tt>execve()</tt>, <tt>execvp()</tt>, <tt>getpid()</tt>.

Note how these are statically linked libraries and are not linked. It will do to simply add function prototypes with no implementation (or better yet, a stub implementation, or a real one) and no linker errors will happen unless the relevant parts of libgcc or libgcov are needed. Note how these are just the minimum requirements to get by without any implicit-function-declaration issues; the code is able to use more standard library features if available.

For the record and convenience, [https://gist.github.com/sortie/9a64aa532052e2137f05f5324a75b474 here are minimal headers] that satisfies the requirements. (UPDATE: This repo is outdated, and GCC will still warn you about missing declarations.  Just add them as you see them, it's only a few, and you can copy from your system headers.)

== Sysroot Headers ==

Your previous <tt>i686-elf</tt> toolchain was built using the <tt>--without-headers</tt> configure option. This tells the compiler that there is no standard library and no headers. The libgcc is built with libc support as described above. This requires the standard library headers to be installed into your [[Meaty Skeleton#System Root|system root]] prior to the cross-compiler build.

<syntaxhighlight lang="bash">
cd $HOME/myos
mkdir -p sysroot
mkdir -p sysroot/usr/include
cp -RT libc/include sysroot/usr/include
cp -RT kernel/include sysroot/usr/include
</syntaxhighlight>

'''Note''': Not all <tt>cp(1)</tt> implementations support the <tt>-RT</tt> combination that conveniently merges the contents of one directory into another.

== Binutils ==

<syntaxhighlight lang="bash">
cd $HOME/src
mkdir build-binutils
cd build-binutils
../myos-binutils/configure --target=i686-myos --prefix="$PREFIX" --with-sysroot=$HOME/myos/sysroot --disable-werror
make
make install
</syntaxhighlight>

'''--with-sysroot=''' This option tells the compiler where your sysroot is. It is not used during the compilation process, but it is remembered and used when the linker searches for libraries.

== GCC ==

<syntaxhighlight lang="bash">
cd $HOME/src
mkdir build-gcc
cd build-gcc
../myos-gcc/configure --target=i686-myos --prefix="$PREFIX" --with-sysroot=$HOME/myos/sysroot --enable-languages=c,c++
make all-gcc all-target-libgcc
make install-gcc install-target-libgcc
</syntaxhighlight>

'''--with-sysroot=''' This option tells the compiler where your sysroot is. It is used during the libgcc build and is remembered and used when the compiler searches for headers and libraries.

== libc ==

You can now compile your [[C Library]] normally using your fresh, custom cross-compiler.

== libstdc++ ==

The libstdc++ library was still not built above as it depends on libc as it needs to perform link tests to know about it. The GCC developers have poorly designed the libstdc++ package and it is tied to GCC and must be built as part of GCC, it can't be built on its own without tricks. If you saved your build-gcc directory from earlier, you can continue the process now (otherwise, it will build a full new cross-compiler for no good reason):

<syntaxhighlight lang="bash">
cd $HOME/src/build-gcc
make all-target-libstdc++-v3
make install-target-libstdc++-v3
</syntaxhighlight>

=== Troubleshooting ===
In case you get an this error (occurs when using newlib):

<pre>
./../../../gcc-11.3.0/libstdc++-v3/libsupc++/new_opa.cc:62:1: error: ‘void* aligned_alloc(std::size_t, std::size_t)’ was declared ‘extern’ and later ‘static’ [-fpermissive]
 aligned_alloc (std::size_t al, std::size_t sz)
</pre>

Open the file '''libstdc++-v3/libsupc++/new_opa.cc''' with an editor and remove the "static inline" before in the function prototype.

== Conclusion ==

You now have a full <tt>i686-myos</tt> toolchain that provides a hosted environment producing executables for your user-space. Your program loader will load programs into the current process, set up an initial thread at the program entry point, and the process will perform system calls to cooperate with the kernel.

You can continue to polish your custom toolchain by teaching it more about your operating system and customizing its behavior to your needs. As your standard library and kernel support improves, you will become able to easily [[Cross-Porting Software|port third party software]] to your operating system by making third party software use your cross-compiler.

== See Also ==
=== Articles ===
* [[GCC]]
* [[GCC Cross-Compiler]]
* [[OS Specific Toolchain]]
* [[ELF]]
* [[System Call]]
* [[Porting GCC to your OS]]
* [[Cross-Porting Software]]

[[Category:Compilers]]
[[Category:Tutorials]]
````

<a id="w003638"></a>

## W003638 — Roll Your Own Filesystem

Bron-ID: W003638
Namespace: Main (0)
Revisie: 30846 | Datum: 2026-07-04T13:13:54Z
Bron: https://wiki.osdev.org/Roll_Your_Own_Filesystem
Revisie-URL: https://wiki.osdev.org/index.php?title=Roll_Your_Own_Filesystem&oldid=30846

````text
{{In Progress}}{{Rating|4}}
{{BadPractice}}
== Introduction ==
'''Please note that rolling your own filesystem IS NOT RECOMMENDED'''. You're likely to make it a FAT style filesystem, and please, we don't need another one of those. However, for those of you who wish to defy convention, then we need to stop you making another FAT for sanity's sake. So here is a tutorial on how to avoid a FAT (of course, the design is up to you, so we can't give you a ''complete'' tutorial on how to make it).

== Use FUSE ==
The best practice for a new file system is to start with a [[FUSE]] driver. This is a simple user space application running on your build machine that allows mounting partition images of your file system. With that, you can easily create files and directories in the image, and later you can re-use significant parts of the code in your kernel when you'll write your native FS implementation. It is much preferable than writing your own image creator.

== Avoiding FAT ==
So how do you avoid a FAT? It all comes down to recognizing a FAT and punching yourself if your FS matches the description.

=== Dos ===
* Use trees
* ''others?''

=== Don'ts ===
* Use a File Allocation Table
* ''others?''

== Booting from your filesystem ==
You have 4 options here. Here are your options, in order of difficulty (and reward)
* Don't boot from it. Use a dedicated boot partition (like [[EFI System Partition]]) and your FS as the root.
* Use [[BOOTBOOT]] which can load an initial ramdisk from any file system.
* [[Rolling Your Own Bootloader | Roll your own bootloader]]
* Use [[GRUB]] by [[Writing GRUB Modules|Writing a GRUB Module]]. You didn't expect GRUB to by psychic and magically code itself, did you?

[[Category:Filesystems]]
````

<a id="w003639"></a>

## W003639 — Writing GRUB Modules

Bron-ID: W003639
Namespace: Main (0)
Revisie: 28139 | Datum: 2023-07-10T02:59:29Z
Bron: https://wiki.osdev.org/Writing_GRUB_Modules
Revisie-URL: https://wiki.osdev.org/index.php?title=Writing_GRUB_Modules&oldid=28139

````text
{{Rating|3}}

The whole architecture of [[GRUB]] changed as of version 2. The big new feature is modules. This lead to the elimination of Stage 1.5. It also means it is easier for you to put lot of cool functionality into your bootloader. You can now add support for new executable formats, filesystems, video drivers, kernel boot protocols, and so on.

== Modules ==
GRUB modules are relocatable ELF32 binaries. They have the extension .mod.

=== Header files ===
All the standard GRUB header files from the source distribution can be included.  If you are building out-of-tree and have not configured the source for your system, then these can still be used with a few modifications:
* define GRUB_MACHINE_PCBIOS to be 1
* define GRUB_UTIL
* define GRUB_FILE to be something appriopriate
* provide include/grub/cpu/memory.h to typedef grub_phys_addr_t to something appropriate (e.g. intptr_t)

=== Special Sections ===
Each module is required to contain three special pieces of information: the module name, the dependencies of the module (i.e. which other modules need to be loaded prior to this one) and the licence of the module.  For GRUB 2, the licence is required to be GPL otherwise it will not be loaded.  These need to be placed in special sections within the module.  The following piece of code (GCC specific) at module scope will achieve the required functionality:
<source lang="C">
char modname[] __attribute__((section(".modname"))) = "mymodule";

/* Example dependencies */
char moddeps[] __attribute__((section(".moddeps"))) = "relocator\0video\0vbe\0multiboot\0gfxterm";

GRUB_MOD_LICENSE("GPLv3+");
</source>

=== Initialization ===
You need to provide initialization and finalization functions for your module.  These are called when the module is loaded/unloaded, and should register the commands that the module provides along with any other resources it requires.  A short example is:
<source lang="C">
static grub_extcmd_t cmd;

GRUB_MOD_INIT(mymodule)
{
    cmd = grub_register_extcmd("mycommand" /* name of the command */,
            grub_cmd_mycommand /* function to call when the command is run */,
            0 /* flags */,
            0 /* summary */,
            N_("In-depth description of this command") /* description */,
            0 /* argument parser */);
}

GRUB_MOD_FINI(mymodule)
{
    grub_unregister_extcmd(cmd);
}
</source>

See the GRUB source/example modules for more information about the other options here.

=== Functions ===
Some useful functions exposed by grub are given below.  Note that there are implementations of typical libc/POSIX functions often prefixed by grub_ in the grub include directory.  Some may require additional modules to be loaded first - GRUB will highlight this when you try and insmod a module that contains functionality that is not currently loaded.

<source lang="C">
int EXPORT_FUNC() grub_printf(const char* func, ...);  //Very similar to standard printf()
void *grub_malloc(grub_size_t size);         // grub's malloc function - memory allocated by this will
                                             //  not necessarily be accessible to your kernel.  See the
                                             //  relocator module if this is the functionality you need.
grub_extcmd_t grub_register_extcmd (const char * name, grub_extcmd_func_t func, 
    grub_command_flags_t flags, const char * summary, const char * description, 
    const struct grub_arg_option * parser);  //Registers a command on the GRUB command line
grub_fs_register (struct grub_fs* fs);       //Registers a filesystem with GRUB (grub_fs is a set of function pointers)
grub_err_t grub_video_set_mode(const char *modestring, unsigned modemask, unsigned modevalue);
                                             // Attempt to set a video mode.
                                             //  e.g. grub_video_set_mode("1024x768x32,auto", 0, 0);
</source>

=== Building ===
Modules are object files. You therefore need to use GCC (i686-elf recommended) to build your C files, then use ld's -r option to combine your object files (if necessary). the final output needs to be named .mod.

A particular register calling convention is used by GRUB, such that the first three parameters are passed in eax, edx and ecx then on the stack, and the callee cleans the stack.  If you use C and the GRUB build system to handle building your module then this is handled for you.  If you build 'out-of-tree' without first configuring GRUB to build for your system, however, then the following GCC flags will be required for compilation (includes the #defines listed above):
<pre>
GRUBCFLAGS = -Os -DGRUB_MACHINE_PCBIOS=1 -falign-jumps=1 -falign-loops=1 -falign-functions=1 \ 
    -mno-mmx -mno-sse -mno-sse2 -mno-3dnow -m32 -fno-stack-protector -mno-stack-arg-probe \
    -mrtd -mregparm=3 -DGRUB_UTIL -ffreestanding -DGRUB_FILE=__FILE__
</pre>

=== Loading ===
Modules can be loaded with the insmod command, from the GRUB configuration file. For filesystem modules, this allows your OS to be booted from your FS, but requires a recognized format for your module. To solve this, you will need to add your module to core.img using grub-mkimage, which can be installed using grub-setup, or possibly grub-install (which is a shell script wrapper to mkimage and setup).

== External Links ==
* [http://blog.fpmurphy.com/2010/06/grub2-modules.html A tutorial by fpmurphy]

[[Category:GRUB]]
[[Category:Bootloaders]]
````

<a id="w003640"></a>

## W003640 — Template:BadPractice

Bron-ID: W003640
Namespace: Template (10)
Revisie: 30609 | Datum: 2026-05-22T14:08:40Z
Bron: https://wiki.osdev.org/Template:BadPractice
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ABadPractice&oldid=30609

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
[[image:LFHSDevsHSHubHSPortIncorrect.gif|50x50px|Proceed with caution!]]
|
This page or section promotes [[Bad Practices|Bad or outdated practice(s)]]. Follow with caution, and help the wiki by [{{fullurl:{{FULLPAGENAME}}|action=edit}} keeping the information up to date].
|
|}
</center>

<includeonly>[[Category:Inadvisable]]</includeonly>
````

<a id="w003641"></a>

## W003641 — Category:Inadvisable

Bron-ID: W003641
Namespace: Category (14)
Revisie: 16800 | Datum: 2014-09-23T22:28:06Z
Bron: https://wiki.osdev.org/Category:Inadvisable
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AInadvisable&oldid=16800

````text
These pages are for people who can't follow convention, and just love breaking all the rules '''that are for their own good'''.
````

<a id="w003642"></a>

## W003642 — User:Gudenau

Bron-ID: W003642
Namespace: User (2)
Revisie: 16805 | Datum: 2014-09-24T15:03:48Z
Bron: https://wiki.osdev.org/User:Gudenau
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGudenau&oldid=16805

````text
I am interested in the inner workings of modern technology, and I am working on a simple os. When I find a dead link, I will try to add a cached copy of it.
````

<a id="w003643"></a>

## W003643 — CompilerDev/Bare Bones Compiler

Bron-ID: W003643
Namespace: Main (0)
Revisie: 28201 | Datum: 2023-07-10T17:58:57Z
Bron: https://wiki.osdev.org/CompilerDev/Bare_Bones_Compiler
Revisie-URL: https://wiki.osdev.org/index.php?title=CompilerDev%2FBare_Bones_Compiler&oldid=28201

````text
{{In Progress}}{{Rating|2}}
[[Compiler]]s are a very important piece of software in any Operating System Developer's toolkit. Without it, your code would only be code. You would have to write all executables by hand, or in assembly, and then assemble it. Compilers make this easy by allowing us to use a higher level language, such as [[C]], [[C++]], or [[D]], in traditionally lower level programming. For example, Operating System development (such as the first builds of unix and apple DOS) used to have to be written in Assembly, which isn't neccesarily very portable, but with the creation of languages such as C, you are able to program in higher level, human-readable code that can be ported to any operating system that has a c compiler for it. Today, we are going to learn how to write a dsl in the C programming language.


'''Disclaimer:''' I am no where near an expert on the matter. This is not for people looking for advanced compiler lessons. This is for people who don't know anything about writing a compiler. Also, the code shown here might not necessarily be the best written, but the theories are tight.

== Required Knowledge: ==
* Knowledge of the C programming language and its standard library
* Basic Algorithim Design
* Ability to read a theory and implement it in code (if you have trouble with this, try practicing on a few [http://goo.gl/Bo4C5m algorithms of varying difficulty])
* Knowledge of the [[ELF]] file structure
* Knowledge of the Unix Operating System

== What This Tutorial Covers ==
== What This Tutorial Doesn't Cover ==

[[Category:Compiler Development]]
````

<a id="w003644"></a>

## W003644 — UDI Call Marshalling and RPC

Bron-ID: W003644
Namespace: Main (0)
Revisie: 16979 | Datum: 2014-10-29T10:23:16Z
Bron: https://wiki.osdev.org/UDI_Call_Marshalling_and_RPC
Revisie-URL: https://wiki.osdev.org/index.php?title=UDI_Call_Marshalling_and_RPC&oldid=16979

````text
[[Category:Uniform Driver Interface]]
[[Category:Drivers]]
[[Category:Driver Interfaces]]

= Prerequisite reading =
UDI Core Specification, Vol 1, Section 28.3: "Marshalling".
UDI Core Specification, Vol 1, Section 28.4: "MEI Stubs".

Documentation for the following service call:
<source lang="c">
void udi_mei_call (
   udi_cb_t *gcb,
   udi_mei_init_t *meta_info,
   udi_index_t meta_ops_num,
   udi_index_t vec_idx,
   ... );
</source>

Documentation surrounding the following UDI abstract types:
<source lang="c">
typedef udi_ubit8_t udi_layout_t;

typedef const struct {
	const char *op_name;
	udi_ubit8_t op_category;
	udi_ubit8_t op_flags;
	udi_index_t meta_cb_num;
	udi_index_t completion_ops_num;
	udi_index_t completion_vec_idx;
	udi_index_t exception_ops_num;
	udi_index_t exception_vec_idx;
	udi_mei_direct_stub_t *direct_stub;
	udi_mei_backend_stub_t *backend_stub;
	udi_layout_t *visible_layout;
	udi_layout_t *marshal_layout;
} udi_mei_op_template_t;
</source>

* [http://uw714doc.sco.com/en/man/html.3udi/udi_mei_op_template_t.3udi.html udi_mei_op_template_t Online reference page].
* [http://uw714doc.sco.com/en/man/html.3udi/udi_layout_t.3udi.html udi_layout_t Online reference page].
* [http://osr600doc.sco.com/en/man/html.UDI/udi_mei_call.UDI.html udi_mei_call() Online reference page].

''Pay particular attention to the <tt>visible_layout</tt> and <tt>marshal_layout</tt> members of <tt>udi_mei_op_template_t</tt> for this article.''

= UDI Call Marshalling =

Call marshalling is the act of taking a function call from a caller, and packaging it such that it can cross one or more boundaries and remain intact and actionable by the callee when it eventually reaches the callee. That is, marshalling a call is the same as a Remote Procedure Call. It is the packaging of a function call such that it can be transported to a callee that is in a different logical environment from the caller. This concept is [http://en.wikipedia.org/wiki/Marshalling_(computer_science) not unique to UDI], and a Google search will yield any number of results talking about the practice.

Reasons why one might need to marshal a UDI call could be:

* Your kernel supports a very complex form of network-based compute clustering, and computers can share the devices on other computers, and use them as their own. In such a case, calls across machines may be very common.
* Your drivers are separate-address-space, much like a microkernel's. You may need to send calls between drivers as messages which will have to cross between address spaces.

UDI recognizes the need for marshalling in microkernels and in distributed computing environments, and standardizes the call-marshalling approach for all UDI calls. ''Marshalling is not usually needed by monolithic kernels since their drivers do not usually call across address spaces''.

For the most part, you will be able to ignore this feature and use it only for determining the total amount of memory required to marshal a call into a message for IPC. However, for very complex Operating Systems which may perform RPC over a network, this can be used to do very cool things, such as '''endianness swapping''' of integers and pointers, etc, for maximum portability. Such a scenario may be necessary if you are on a big-endian machine, and you are sending data to a little endian machine, for example. UDI's design extends gracefully and naturally to very large, high-scaling environments.

== What provisions does UDI make for Call Marshalling? ==

UDI standardizes the marshalling format for UDI Channel (aka, IPC) calls. In essence, the environment shall marshal calls according to the UDI specification if it expects to be sending channel calls across machines with Operating System software that does not match its own.

While it is best practice to use the UDI marshalling format for internal channel IPC between drivers within the same running Operating System, if you choose to use some other marshalling methodology, as long as your calls do not need to leave your own machine (and go across say, a distributed compute cluster), you should not encounter any problems -- but you'll have to do some extra work to call into the target driver on the callee side because of your choice. I will not be going into that in detail, because deviances from the specification are beyond the scope of the article.

== How are calls to be marshalled in a UDI compliant manner? ==

When a microkernel or distributed compute node is marshalling IPC or RPC, it should marshal them into a block of memory whose alignment matches '''the natural data structure alignment of a C compiler'''. The Generic Control Block is expected to be marshalled first; following this should be the Metalanguage-specific portion of the control block. Next, the function call arguments should be packed in after the completed control block, and finally any inline objects should come last.

If the receiving (callee) end of the IPC/RPC transaction is also UDI compliant, it will be able to successfully unpack the call and process it.

== Control Block Marshalling ==

If a function looks like this:
<source lang="c">
void udi_enumerate_ack(
	udi_enumerate_cb_t *cb,
	udi_ubit8_t enumeration_result,
	udi_index_t ops_idx);
</source>

And its control block looks like this:
<source lang="c">
typedef struct {
	udi_cb_t gcb;
	udi_ubit32_t child_ID;
	void *child_data;
	udi_instance_attr_list_t *attr_list;
	udi_ubit8_t attr_valid_length;
	const udi_filter_element_t *filter_list;
	udi_ubit8_t filter_list_length;
	udi_ubit8_t parent_ID;
} udi_enumerate_cb_t;
</source>

The Environment might marshal the call as follows: it would calculate the amount of memory required to hold all of the parameters for the IPC/RPC call, then allocate a block of memory, or an IPC message that could hold all of the parameters. The marshalled call would end up looking like this, if the environment that marshalled it is UDI compliant:
<source lang="c">
struct
{
	udi_cb_t gcb;
	// End of the Generic Control Block.
	udi_ubit32_t child_ID;
	void *child_data;
	udi_instance_attr_list_t *attr_list;
	udi_ubit8_t attr_valid_length;
	const udi_filter_element_t *filter_list;
	udi_ubit8_t filter_list_length;
	udi_ubit8_t parent_ID;
	// End of the METALANGUAGE VISIBLE portion of the control block.
	udi_ubit8_t enumeration_result,
	udi_index_t ops_idx);
	/* End of the function call arguments.
	 *
	 * From here on the environment will copy all of the data pointed to by
	 * call. For example,
	 *	udi_instance_attr_list_t *attr_list;
	 *
	 * This "attr_list" parameter points to an array of objects which will
	 * need to be read by the callee. If this array is not also marshalled
	 * and transmitted, the callee will be lacking a part of the call.
	 * So following the function call arguments, we now begin marshalling
	 * these "inline objects" that are pointed to by the procedure call.
	 *
	 * UDI refers to such objects as "Inline objects", because they are
	 * "out of line" with the rest of the procedure call, but pointed
	 * to by pointers which are "inline" in the control block, and needed
	 * by the callee.
	 */
	<COPY HERE THE OBJECT POINTED TO BY "child_data">
	<COPY HERE THE ARRAY FOR "attr_list">
	// End of "attr_list" array.
	<COPY HERE THE ARRAY FOR "filter_list">
} marshalled_udi_enumerate_ack_call;
</source>

= UDI Layout Descriptors =

The key UDI data type that will be discussed in this section is [http://uw714doc.sco.com/en/man/html.3udi/udi_layout_t.3udi.html udi_layout_t]:
<source lang="c">
typedef udi_ubit8_t udi_layout_t;
</source>

The most obvious question that will be asked is, "How could the environment know how many arguments are in a function call? Or how large the data pointed to by it is?"

Because of the strictly defined interfaces in UDI, known as [[UDI Metalanguages|Metalanguages]], all communications across UDI channels are well defined. And the Metalanguage libraries for each Metalanguage protocol also describe the function calls that are sent across them.

We will continue using the example from above, with the <tt>udi_enumerate_ack</tt> call. The <tt>udi_enumerate_ack</tt> call has:

* A Generic Control Block (like every other function in UDI).
* A set of Metalanguage '''visible''' members in its control block.
* And function call arguments passed on the stack.

We saw how these were marshalled into the designated memory above. In the metalanguage library that defines the function "<tt>udi_enumerate_ack</tt>", there is an array of descriptors that gives the environment the layout of the parameters taken by <tt>udi_enumerate_ack</tt>. It looks something like this:

<source lang="c">
static udi_layout_t		udi_enumerate_ack_visible_layout[] =
{
	UDI_DL_UBIT32_T, UDI_DL_INLINE_UNTYPED,
	UDI_DL_MOVABLE_UNTYPED, UDI_DL_UBIT8_T,
	UDI_DL_MOVABLE_UNTYPED, UDI_DL_UBIT8_T,
	UDI_DL_UBIT8_T,
	UDI_DL_END
};

const static udi_layout_t		udi_enumerate_ack_marshal_layout[] =
	{ UDI_DL_UBIT8_T, UDI_DL_INDEX_T, UDI_DL_END };
</source>

The <tt>udi_enumerate_ack_'''visible_layout'''</tt> describes the Metalanguage visible portion of the control block. The <tt>udi_enumerate_ack_'''marshal_layout'''</tt> describes the stack arguments. ''There are no descriptors anywhere for the Generic Control block portion of every call, because it is well known and never needs to be described. Generic Control Blocks have the same structure.''

In the real world, these two layout descriptors are found within the <tt>'''udi_mei_op_template_t'''</tt> entry for that function.

Each entry (UDI_DL_UBIT32_T, UDI_DL_INLINE_UNTYPED, etc) tells the environment how large the function parameter is expected to be. The entries are listed in order, so the ordering is also known to the environment. Using this information, the environment can safely marshal calls across various types of boundaries in a portable manner for any type of IPC or RPC operation.
````

<a id="w003645"></a>

## W003645 — Template:Navigation

Bron-ID: W003645
Namespace: Template (10)
Revisie: 16843 | Datum: 2014-10-03T11:11:50Z
Bron: https://wiki.osdev.org/Template:Navigation
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ANavigation&oldid=16843

````text
<table style="width: 100%; background-color: #BDC3C7; border: 1px solid #96A6A7;">
<tr>
<td style="width: 30%; text-align: left;">{{if | {{{back|}}} | &laquo;{{{back|}}} }}</td>
<td style="width: 40%; text-align: center;">{{if | {{{title|}}} | '''{{{title|}}}''' | '''navigation''' }}</td>
<td style="width: 30%; text-align: right;">{{if | {{{forward|}}} | {{{forward|}}}&raquo; }}</td>
</tr>
</table>
````

<a id="w003649"></a>

## W003649 — Talk:D Bare Bones

Bron-ID: W003649
Namespace: Talk (1)
Revisie: 20739 | Datum: 2017-04-22T14:51:08Z
Bron: https://wiki.osdev.org/Talk:D_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AD_Bare_Bones&oldid=20739

````text
== Cross-Compiler ==

Is a CrossCompiler needed?
[[User:Columbus|Columbus]] 08:03, 5 October 2014 (CDT)

If not, how are the commands on x86_64, or in other words, the most specific commands with march and so on?
[[User:Columbus|Columbus]] 10:33, 20 January 2015 (CST)

== volatile keyword not allowed ==

The volatile keyword in D is beyond deprecated; i.e. not allowed anymore. The code in this page must be updated.
--[[User:Robbie0630|Robbie0630]] 09:51, 22 April 2017 (CDT)
````

<a id="w003650"></a>

## W003650 — User:Gravaera/UDI Environment Implementation Guide

Bron-ID: W003650
Namespace: User (2)
Revisie: 16864 | Datum: 2014-10-06T21:10:13Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Environment_Implementation_Guide
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Environment_Implementation_Guide&oldid=16864

````text
= Preamble =

Summary of the article, why it is being written, whom it will be helpful to, what level of detail is to be expected, and what level of abstraction is to be expected in the writing style of the article.

= Order of prescribed implementation =

Quick explanation for why this ordering is recommended.

== Database of supported devices ==
Quick explanation of the work entailed for each milestone.
== Driver matching service ==
Quick explanation of the work entailed for each milestone.
== Skeleton driver instances ==
Quick explanation of the work entailed for each milestone.
== Metadata compilation ==
Quick explanation of the work entailed for each milestone.
== Regions ==
Quick explanation of the work entailed for each milestone.
== Channels ==
Quick explanation of the work entailed for each milestone.
== Channel IPC calls (udi_mei_call()) ==
Quick explanation of the work entailed for each milestone.
== UDI Service Calls (a.k.a. UDI related syscalls) ==
Quick explanation of the work entailed for each milestone.

Below here, one section for each milestone of work.
````

<a id="w003651"></a>

## W003651 — Talk:RSDP

Bron-ID: W003651
Namespace: Talk (1)
Revisie: 30214 | Datum: 2026-03-13T15:13:19Z
Bron: https://wiki.osdev.org/Talk:RSDP
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ARSDP&oldid=30214

````text
Shouldn't there be an "'''__attribute__((packed))'''" on these struct definitions? Or at least an info for compilers other than GCC. :) --[[User:Max|Max]] 15:01, 7 October 2014 (CDT)

Correct me if i'm wrong, but should this article be marked with the non-encyclopedic tone thing? I noticed things like use of AFAIK and such --[[User:AlulaOneshot|AlulaOneshot]] 7:51, 13 March 2026 (EST).
: Yup, also a lot of usage of "you", feel free to reword the article if you would --[[User:Tocariimaa|Tocariimaa]] ([[User talk:Tocariimaa|talk]]) 15:13, 13 March 2026 (UTC)
````

<a id="w003652"></a>

## W003652 — Talk:MADT

Bron-ID: W003652
Namespace: Talk (1)
Revisie: 16873 | Datum: 2014-10-08T21:50:46Z
Bron: https://wiki.osdev.org/Talk:MADT
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMADT&oldid=16873

````text
I noticed that this page is mostly incomplete and doesn't contain any helpful information that the ACPI specification does not contain - does it make sense to keep such a page? Or rather link to the ACPI specification? --[[User:Max|Max]] 16:50, 8 October 2014 (CDT)
````

<a id="w003656"></a>

## W003656 — User:Huntnb

Bron-ID: W003656
Namespace: User (2)
Revisie: 21504 | Datum: 2017-09-10T16:12:02Z
Bron: https://wiki.osdev.org/User:Huntnb
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHuntnb&oldid=21504

````text
= Memory Management =

Four different memory managers are needed:

* Physical page frame management (contiguous regions in physical memory)
* Kernel-space byte-sized allocator (<code>kmalloc()</code>/<code>kfree()</code>)
* SLAB-style allocator for frequently created/destroyed objects
* User-space byte-sized allocator (<code>malloc()</code>/<code>free()</code>)

== Physical Frame Management ==

The physical frame manager keeps track of which physical frames are currently in use. During kernel initialization, the system needs to count the available memory, then classify each 4KiB frame as either "in-use" or "free". The API for the physical frame manager might look something like:

* <code>paddr_t frame_alloc(..., int count)</code>
* <code>paddr_t frame_free(paddr_t frame)</code>
* <code>void frame_mark_in_use(paddr_t start, int size)</code>
* <code>void frame_mark_free(paddr_t start, int size)</code>

=== Bitmap Allocator ===

For every frame in physical memory, keep track of a single bit that is set when the frame is in use, and cleared when it is available. Allocating a page is a linear search to find the first cleared bit, and freeing a frame is constant time.

=== Stack Allocator ===

For every physical page available for use in the system, push it on to the end of a "free list".  As pages are needed, they can be pulled from the front of the list in constant time.  Similarly, when a frame becomes free, it is linked to the beginning of the free list.

=== Buddy Allocator ===

Good reference [http://en.wikipedia.org/wiki/Buddy_memory_allocation here].

== Kernel Byte-sized Allocator ==

== SLAB Object Cache ==

For each object type that is frequently created/destroyed, create a new "cache" object for that type. The cache maintains three separate linked lists: <code>full</code>, <code>partial</code>, and <code>empty</code>. Each is a list of allocated <code>slab</code> objects. A slab (typically 1 frame in size) contains as many pre-allocated objects of the cache's type that will fit. Book keeping is needed to keep track of which objects on a slab are currently in use and which are free. If all objects in a slab are allocated, the slab is on the <code>full</code> list; if none are allocated, it is on the <code>empty</code> list; otherwise, it is on the <code>partial</code> list. When requesting a new object from the cache, it first attempts to satisfy the request from a <code>partial</code> slab. If no such slab is found, an <code>empty</code> slab is used. If no <code>empty</code> slab is available, a new slab is requested from the kernel.

Occasionally, caches can be reaped. If memory is low, caches can release their <code>empty</code> pages back to the page pool. Coalescing <code>partial</code> slabs is probably not feasible: pointers to slab objects can be located anywhere in memory and would need to be updated if the object is moved from one slab to another.

== User Byte-sized Allocator ==

Should be very similar in design and implementation as the kernel's version.
````

<a id="w003657"></a>

## W003657 — GCC Stack Smashing Protector

Bron-ID: W003657
Namespace: Main (0)
Revisie: 16949 | Datum: 2014-10-22T23:28:49Z
Bron: https://wiki.osdev.org/GCC_Stack_Smashing_Protector
Revisie-URL: https://wiki.osdev.org/index.php?title=GCC_Stack_Smashing_Protector&oldid=16949
Doorverwijzing: Stack Smashing Protector

````text
#REDIRECT [[Stack Smashing Protector]]
````

<a id="w003658"></a>

## W003658 — Talk:GCC Stack Smashing Protector

Bron-ID: W003658
Namespace: Talk (1)
Revisie: 16951 | Datum: 2014-10-22T23:28:49Z
Bron: https://wiki.osdev.org/Talk:GCC_Stack_Smashing_Protector
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGCC_Stack_Smashing_Protector&oldid=16951
Doorverwijzing: Talk:Stack Smashing Protector

````text
#REDIRECT [[Talk:Stack Smashing Protector]]
````

<a id="w003659"></a>

## W003659 — Talk:Libgcc without red zone

Bron-ID: W003659
Namespace: Talk (1)
Revisie: 16964 | Datum: 2014-10-23T19:59:22Z
Bron: https://wiki.osdev.org/Talk:Libgcc_without_red_zone
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALibgcc_without_red_zone&oldid=16964

````text

== For a userland-less GCC, isn't this normally done with <code>CFLAGS_FOR_TARGET</code>? ==

Given that a <code>x86_64-pc-elf</code> GCC is mostly useless for userland programming (without -mno-red-zone), shouldn't it just be built with <code>CFLAGS_FOR_TARGET=-mno-red-zone</code>? See this gcc-help posting: [https://gcc.gnu.org/ml/gcc-help/2011-07/msg00158.html]. Also, it'd be interesting to know how this is achieved in Linux, *BSD, &c. kernels that are built with a conventional userland compiler and don't have (?) a special libgcc. --[[User:Alexshpilkin]] (Forgot to sign message)
:Thank's for bringing the topic up. I tried <code>CFLAGS_FOR_TARGET</code> (as that flag is documented by gcc) but it's silently ignored when building the <code>crtbegin</code> and <code>crtend</code> objects. Trust me with this. It took about three days to figure out a clean way to get the <code>-mno-redzone</code> to apply to these files. We also had lots of discussion on that topic in #osdev. --[[User:Shirk|Shirk]] 12:00, 23 October 2014 (CDT)
````

<a id="w003660"></a>

## W003660 — User:Huntnb/List

Bron-ID: W003660
Namespace: User (2)
Revisie: 16962 | Datum: 2014-10-23T17:14:48Z
Bron: https://wiki.osdev.org/User:Huntnb/List
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AHuntnb%2FList&oldid=16962

````text
= Generic List Implementation =

See Linux's [http://lxr.free-electrons.com/source/include/linux/list.h list.h] or FreeBSD's [http://akaros.cs.berkeley.edu/lxr/#akaros/kern/include/sys/queue.h queue.h] for tried-and-true production implementations.
````

<a id="w003661"></a>

## W003661 — Strings do not work

Bron-ID: W003661
Namespace: Main (0)
Revisie: 25273 | Datum: 2020-10-28T20:10:00Z
Bron: https://wiki.osdev.org/Strings_do_not_work
Revisie-URL: https://wiki.osdev.org/index.php?title=Strings_do_not_work&oldid=25273
Doorverwijzing: Bran's Kernel Development Tutorial Known Bugs

````text
#REDIRECT [[Bran's_Kernel_Development_Tutorial_Known_Bugs#As soon as I add strings, things go wrong.]]
[[Category:FAQ]]
````

<a id="w003662"></a>

## W003662 — User:Lisovy

Bron-ID: W003662
Namespace: User (2)
Revisie: 16968 | Datum: 2014-10-27T11:12:21Z
Bron: https://wiki.osdev.org/User:Lisovy
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ALisovy&oldid=16968

````text
== Rostislav Lisovy ==
lisovy@gmail.com
````

<a id="w003663"></a>

## W003663 — User:Gravaera/UDI Marshalling Layouts and Inline Control Block Elements

Bron-ID: W003663
Namespace: User (2)
Revisie: 16976 | Datum: 2014-10-29T04:54:11Z
Bron: https://wiki.osdev.org/User:Gravaera/UDI_Marshalling_Layouts_and_Inline_Control_Block_Elements
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AGravaera%2FUDI_Marshalling_Layouts_and_Inline_Control_Block_Elements&oldid=16976
Doorverwijzing: UDI Call Marshalling and RPC

````text
#REDIRECT [[UDI Call Marshalling and RPC]]
````

<a id="w003664"></a>

## W003664 — User:Stephanvanschaik/Setting Up Long Mode

Bron-ID: W003664
Namespace: User (2)
Revisie: 16986 | Datum: 2014-11-01T18:57:53Z
Bron: https://wiki.osdev.org/User:Stephanvanschaik/Setting_Up_Long_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AStephanvanschaik%2FSetting_Up_Long_Mode&oldid=16986
Doorverwijzing: Setting Up Long Mode

````text
#REDIRECT [[Setting Up Long Mode]]
````

<a id="w003665"></a>

## W003665 — User talk:Stephanvanschaik/Setting Up Long Mode

Bron-ID: W003665
Namespace: User talk (3)
Revisie: 16988 | Datum: 2014-11-01T18:57:54Z
Bron: https://wiki.osdev.org/User_talk:Stephanvanschaik/Setting_Up_Long_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AStephanvanschaik%2FSetting_Up_Long_Mode&oldid=16988
Doorverwijzing: Talk:Setting Up Long Mode

````text
#REDIRECT [[Talk:Setting Up Long Mode]]
````

<a id="w003666"></a>

## W003666 — Pmode

Bron-ID: W003666
Namespace: Main (0)
Revisie: 17006 | Datum: 2014-11-09T10:30:48Z
Bron: https://wiki.osdev.org/Pmode
Revisie-URL: https://wiki.osdev.org/index.php?title=Pmode&oldid=17006
Doorverwijzing: Protected Mode

````text
#REDIRECT [[Protected Mode]]
````

<a id="w003667"></a>

## W003667 — User:Columbus

Bron-ID: W003667
Namespace: User (2)
Revisie: 17077 | Datum: 2014-11-18T18:19:59Z
Bron: https://wiki.osdev.org/User:Columbus
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AColumbus&oldid=17077

````text
Hi.

I've created a Tutorial on how to compile a or1k Toolchain. [[User:Columbus/OR1K_Toolchain|here]]

I also made a Tutorial, on a or1k conversion of the [[Bare Bones]]. [[User:Columbus/OR1K_BareBones|here]]
````

<a id="w003668"></a>

## W003668 — User:Columbus/OR1K Toolchain

Bron-ID: W003668
Namespace: User (2)
Revisie: 17076 | Datum: 2014-11-18T16:28:48Z
Bron: https://wiki.osdev.org/User:Columbus/OR1K_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AColumbus%2FOR1K_Toolchain&oldid=17076

````text
A tutorial, how to build a OpenRISC toolchain.
At the end of the tutorial, you have a GCC-Toolchain for or1k. I'm trying to use it for OS deving.
I'd be happy, if someone could write a Barebone for it, because I don't understand the or1k opcodes/mnemonics.

== Prerequisites ==
You must have ''git'' installed, and the folder ''/opt/or1k-toolchain'' should exist, and the user you're using should have write access.

== Compiling the toolchain ==
The toolchain gets installed to ''/opt/or1k-toolchain''
In some directory do

<source lang="bash"># Clone repos
git clone git://github.com/openrisc/or1k-src.git
git clone git://github.com/openrisc/or1k-gcc.git
# Create build directories
mkdir build-or1k-src build-or1k-gcc
# Extend $PATH
export PATH=$PATH:/opt/or1k-toolchain

# Build src
cd build-or1k-src

../or1k-src/configure --target=or1k-elf      \
--prefix=/opt/or1k-toolchain --enable-shared \
--disable-itcl   --disable-tk --disable-tcl  \
--disable-winsup --disable-gdbtk             \
--disable-libgui --disable-rda --disable-sid \
--disable-sim    --disable-gdb --with-sysroot \
--disable-newlib --disable-libgloss          \
--disable-werror

make -j8
make install

# Build gcc
cd ../bld-or1k-gcc

../or1k-gcc/configure --target=or1k-elf           \
--prefix=/opt/or1k-toolchain --enable-languages=c \
--disable-shared --disable-libssp --disable-werror

make -j8
make install -j8

# Rebuild src with newlib
cd ../bld-or1k-src
rm -rf *

../or1k-src/configure --target=or1k-elf      \
--prefix=/opt/or1k-toolchain --enable-shared \
--disable-itcl   --disable-tk --disable-tcl  \
--disable-winsup --disable-gdbtk --disable-nls \
--disable-libgui --disable-rda --disable-sid \
--enable-sim  --disable-or1ksim --enable-gdb \
--with-sysroot   --enable-newlib             \
--enable-libgloss --disable-werror

make -j8
make install -j8

# Rebuild gcc with newlib
cd ../bld-or1k-gcc
rm -rf *

../or1k-gcc/configure --target=or1k-elf               \
--prefix=/opt/or1k-toolchain --enable-languages=c,c++ \
--disable-shared --disable-libssp --with-newlib       \
--disable-werror --disable-nls --without-headers

make -j8
make install -j8
</source>

== Further ==
You now have a C/C++ Compiler, and an Assembler with or1k as target.
But you should know, that in the Assembler, you can only use [[GAS]]-Keywords, and or1k instructions.
If you don't have problems with that, have fun with linking and so on as usual.

I found an example of or1k-Assembly code. It's the asm code of the libc implementation of the or1k toolchain. Its [http://opencores.org/websvn,filedetails?repname=openrisc&path=%2Fopenrisc%2Ftrunk%2Fgnu-stable%2Fnewlib-1.18.0%2Flibgloss%2For32%2Fcrt0.S here].

One could also combine with the GDC-Toolchain tutorial, to have a really special thing. I didn't try this, and I don't know if or1k breaks GDC

If you're interested, the or1k page is [http://opencores.org/or1k/Main_Page here]. It contains the Architecture Specification, which names the instructions that you can use.

== Sources ==
I got the main commands from the [http://opencores.org/or1k/OpenRISC_GNU_tool_chain#Installation_of_development_versions| or1k Documentation]

Then I combined them with the GCC-CrossCompiler tutorials.
````

<a id="w003669"></a>

## W003669 — Formatted Printing

Bron-ID: W003669
Namespace: Main (0)
Revisie: 30406 | Datum: 2026-03-24T22:11:31Z
Bron: https://wiki.osdev.org/Formatted_Printing
Revisie-URL: https://wiki.osdev.org/index.php?title=Formatted_Printing&oldid=30406

````text
This article will cover the theory over formatted printing (i.e. ''printf()'').

== How ''printf()'' Works? ==
Your first step towards a reliable ''printf()'' function is the understanding of its work. Had you ever tried to imitate ''va_list'' and friends via a ''void**'' taking the example of a specific language, ...
‎General principles and goals - ‎Localising - ‎Translation tools - ‎Babel
Manual:Language - or similitar technics? It surely won't work! You '''must''' use the ''va_list'' provided in the freestanding version of ''<stdarg.h>'' (or and equivalent implementation) in order to be able to use variadic functions. Assuming the use of ''<stdarg.h>'', this is the prototype for ''printf()''
<syntaxhighlight lang="c">
int printf(const char *format, ...);
</syntaxhighlight>

Have you seen the other members of the ''printf()'' family? They're
* ''printf()'': Write formatted data to '''stdout'''
* ''fprintf()'': Write formatted data to stream
* ''sprintf()'': Write formatted data to buffer
* ''snprintf()'': Write formatted data to buffer, limits maximum characters written
* ''vprintf()'': Write formatted data to '''stdout''', uses a '''va_list''' instead of '''...'''
* ''vfprintf()'': Write formatted data to stream, uses a '''va_list''' instead of '''...'''
* ''vsprintf()'': Write formatted data to buffer, uses a '''va_list''' instead of '''...'''
* ''vsnprintf()'': Write formatted data to buffer, limits maximum characters written, uses a '''va_list''' instead of '''...'''

And the abstractions...
* ''printf()'': Wraps over ''vprintf()''
* ''fprintf()'': Wraps over ''vfprintf()''
* ''sprintf()'': Wraps over ''vsprintf()''
* ''snprintf()'': Wraps over ''vsnprintf()''
* ''vprintf()'': Wraps over ''vfprintf()''
* ''vfprintf()'': Does the work!
* ''vsprintf()'': Wraps over ''vsnprintf()'' (with a magical number as limit).
* ''vsnprintf()'': Wraps over ''vfprintf()'' with ''fmemopen()''

But, hey! This is a user-space C library example! Yes, it is. You'll more than likely only provide a ''printf()'' that does what ''vfprintf()'' is intented to do, but in one shot. Basically, the worker function will print where it must print, until finding a '''%''' character. If it's followed by another '''%''', it's printed and process contiues. If it's followed by a valid specifier character, its parsing begins. Else the character is printed and process continues. We'll explain in detail the implementation of each item later.

== Decide on your format ==
Before you begin writing any useful code that processes anything, decide on the format of your format string, and how you want to save the information, if at all (instead opting for printing immediately). Simple implementations should generally work with a format that requires no forward look-ahead and is generally simple to use. You might opt for something seen in more modern languages, with opening and closing curly brackets, as that could be easier to process instead of programming a bunch of logic dictating when, and under which circumstances, you should resume printing as normal (as would be seen if you went with a more standard percent sign based approach, seen in libc).

It might also be beneficial to decide where your output is going to go. Do you want the caller to supply a buffer? Should the function return a pointer to an allocated buffer? Should the caller provide a struct with a pointer to a buffer, the size of said buffer, and a function pointer that accepts that buffer? The last is more flexible and really isn't that hard to implement. You could implement a Writer like the below:

<syntaxhighlight lang="c">
typedef int (*pfnStreamWriteBuf)(char*);

struct Stream {
    size_t buf_len;
    size_t buf_i;
    char *buf;
    pfnStreamWriteBuf pfn_write_all;
};
</syntaxhighlight>

== Implementation ==
This is a not a tutorial, it aims only to guide you through creating your own format or implementing an existing one.

There are two ways to go about this, generally. The first is iterating through the characters, checking each one as you go, and setting a state (a la a state machine). This is probably the easiest option, but it makes it hard to implement looking ahead for more information in the format string without tacking on some code that may or may not have cohesion with the rest of your code.

The other option is focusing on looking ahead once you encounter a specific character that starts your format specifier. This is a bit more complex, and is left as an exercise to the reader. Note that ''this option isn't necessarily better.'' A simpler format generally results in simpler and shorter code, reducing the chance for bugs.

For the first option, initially, you should loop through the format string, looking for the character that triggers your format specifier. Once you encounter one, store that you are currently in "format specifier" mode. If you have a character to end a format specifier, add that code to which switches to "normal" mode.

<syntaxhighlight lang="c">
char current = format_string[i];

switch (current) {
case '{':
    state = FORMAT_SPECIFIER;
    break;
case '}':
    state = NORMAL;
    break;
}
</syntaxhighlight>

You might want to be able to escape the character which enters you into "format specifier" mode:

<syntaxhighlight lang="c">
case '{':
    if (state == NORMAL) {
        state = FORMAT_SPECIFIER;
    } else if (state == FORMAT_SPECIFIER) {
        state = NORMAL;
    }

    break;
</syntaxhighlight>

If your format has a way to specify which type of data should be printed there, you should add this now:

<syntaxhighlight lang="c">
case 'd':
    // Print a digit.
case 's':
    // Print a string.
</syntaxhighlight>

If using C, you should probably accept your arguments through a variadic function, and use the va_arg function to get that data to be printed. With that data, preform any casting or conversion that needs to happen inside the switch statement blocks, and finally print.

From here, it should be fairly simple to add the ability to specify the base with which to print digits with, printing other types of data, and more! You could even add a format specifier which accepts a struct that contains a pointer to another structure and a function pointer which takes that structure and returns back a stringified version of it. This could be handy for printing complex data structures.

That is a rough overview. Check out some example implementation for code.

== Example Implementation ==
A more libc conformant implementation that uses kernel/OS provided capability (not bare metal capable).
[[User:A22347/Printf]]

A custom format for simple code that can run on bare metal using buffered I/O that flushes to a stream.
[[User:Isaccbarker/String_Formatting]]

== See Also ==
=== Articles ===
* [[Printing To Screen]]

[[Category:Tutorials]]
[[Category:Video]]
[[Category:Text UI]]
````

<a id="w003670"></a>

## W003670 — User:Arto

Bron-ID: W003670
Namespace: User (2)
Revisie: 17098 | Datum: 2014-11-23T05:25:32Z
Bron: https://wiki.osdev.org/User:Arto
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AArto&oldid=17098

````text
[http://ar.to/ I] began OS development at the wee age of 13, with Richard Burgess's classic book ''"Developing Your Own 32-Bit Operating System"''. Thus began a long journey that I can't say has reached any particular conclusion, but has certainly been rewarding and educational nonetheless.

These days I do mostly systems programming in C++14. Database engines and such. In my copious free time, I continue working on the tooling and materials that will eventually enable me to put together a complete system according to my own vision.

== My Projects ==

* [http://libc11.org/ libc11], a public domain implementation of the C11 standard library.
* [http://unlicense.org/ The Unlicense], an effort to promote the proliferation of software unfettered and unencumbered by copyright monopoly.

== Elsewhere ==

I am user [http://forum.osdev.org/memberlist.php?mode=viewprofile&u=14370 Arto] on the forum.
````

<a id="w003671"></a>

## W003671 — Cooperative Multitasking

Bron-ID: W003671
Namespace: Main (0)
Revisie: 30768 | Datum: 2026-06-16T15:05:25Z
Bron: https://wiki.osdev.org/Cooperative_Multitasking
Revisie-URL: https://wiki.osdev.org/index.php?title=Cooperative_Multitasking&oldid=30768

````text
{{Stub}}

== See Also ==
=== Articles ===
* [[X86 Cooperative Multitasking Tutorial]]

[[Category:Multitasking]]
[[Category:Task Models]]
````

<a id="w003672"></a>

## W003672 — User:Sortie/Sortix Thread Local Storage

Bron-ID: W003672
Namespace: User (2)
Revisie: 19057 | Datum: 2016-04-08T23:34:19Z
Bron: https://wiki.osdev.org/User:Sortie/Sortix_Thread_Local_Storage
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie%2FSortix_Thread_Local_Storage&oldid=19057

````text
My [https://sortix.org Sortix] operating system has an implementation of [[Thread Local Storage]], these are notes on it:

Each thread has a user-space thread structure (struct pthread) which is used by the pthread implementation. The first member of that structure is a struct uthread, which is part of the kernel interface, and filled in by the kernel for the main thread. The first member of struct uthread is a pointer to itself as this is required on some architecturtes. The struct uthread also contains the location and size of the thread's stack; the location and size of the per-thread allocation; the location, size and alignment of the master thread local storage copy, the location and size of the argument vector allocation (argv and envp passed to main), and flags such as whether this is the first thread.

The rest of the struct pthread is exclusively used by the pthread implementation for mechanisms such as joining and detaching threads.

On i386 and x86-64, the base of the fs and gs segments are considered user-space registers and are set when creating a new thread and can be accessed/changed by the <tt>rdmsr()</tt> and <tt>wrmsr()</tt> system calls.

The kernel sets up thread local storage for the main thread during the <tt>execve()</tt> system call. It locates and loads the master thread local storage somewhere. It creates creates the per-thread allocation somewhere and sets up the thread local storage there, as well as the initializng the struct pthread (by initializing the uthread there, and zeroing the rest of the struct pthread, it finds the size of the struct pthread in the struct uthread). The thread local storage is thus operational immediately at program startup.

The errno lvalue is implemented as <tt>__thread int errno;</tt>.

== Code ==
* [https://gitlab.com/sortix/sortix/blob/master/kernel/include/sortix/uthread.h#L39 struct uthread]
* [https://gitlab.com/sortix/sortix/blob/master/libc/include/pthread.h#L135 struct pthread]
* [https://gitlab.com/sortix/sortix/blob/master/libc/init/init.c#L37 How the program tells the kernel the size of struct pthread]
* [https://gitlab.com/sortix/sortix/blob/master/kernel/elf.cpp#L93 The kernel locates the master TLS copy]
* [https://gitlab.com/sortix/sortix/blob/master/kernel/elf.cpp#L101 The kernel locates the size of struct pthread]
* [https://gitlab.com/sortix/sortix/blob/master/kernel/process.cpp#L839 The kernel creates the main thread at the end of program loading]
* [https://gitlab.com/sortix/sortix/blob/master/kernel/scheduler.cpp#L101 The kernel switches fsbase and gsbase during context switching]
* [https://gitlab.com/sortix/sortix/blob/master/libc/init/init.c#L68 struct pthread is finalized during early program startup]
* [https://gitlab.com/sortix/sortix/blob/master/libc/pthread/pthread_self.c#L22 <tt>pthread_self()</tt>]
* [https://gitlab.com/sortix/sortix/blob/master/libc/pthread/pthread_create.c#L121 <tt>pthread_create()</tt>]
* [https://gitlab.com/sortix/sortix/blob/master/libc/pthread/pthread_exit.c#L28 <tt>pthread_exit()</tt>]
* [https://gitlab.com/sortix/sortix/blob/master/libc/pthread/pthread_join.c#L26 <tt>pthread_join()</tt>]
````

<a id="w003673"></a>

## W003673 — Thread Local Storage

Bron-ID: W003673
Namespace: Main (0)
Revisie: 17081 | Datum: 2014-11-18T20:50:43Z
Bron: https://wiki.osdev.org/Thread_Local_Storage
Revisie-URL: https://wiki.osdev.org/index.php?title=Thread_Local_Storage&oldid=17081

````text
Thread Local Storage (TLS) are per-thread global variables. Compilers such as [[GCC]] provide a <tt>__thread</tt> keyword to mark global variables as per-thread. Support is required in the program loader and thread creator.
<source lang="c">
__thread int errno;
int get_errno() { return errno; }
</source>
A [[x86-64]] [[System V ABI]] compiler would compile this code into assembly like this:
<pre>
	.globl	errno
	.section	.tbss,"awT",@nobits
	.align 4
	.type	errno, @object
	.size	errno, 4
errno:
	.zero	4
...
	movl	%fs:errno@tpoff, %eax
</pre>
The <tt>errno</tt> global variable is put into a special thread local bss section (.tbss) (or .tdata if initialized) and special actions occur at program link time and program load time. A per-thread allocation (containing the thread local storage, the user-space thread structure and perhaps other things) is made when the thread is created. Each per-thread variable is located at a fixed offset inside this allocation. In the above example, the %fs segment starts at the thread's user-space thread structure (%fs thus works as an extra register), and the special <tt>errno@tpoff</tt> linker symbol is the offset from the thread's userspace thread structure to the per-thread errno value.

== Design ==

=== Master TLS Copy ===

The program contains a master copy of its thread local storage (with its initialized-at-compile-time values) which is used when creating threads. This special segment is created by the linker from the <tt>.tdata</tt> (initialized tls) and <tt>.tbss</tt> (zero-initialized tls) sections. You can find it by searching the [[ELF]] program headers for a segment with type <tt>PT_TLS</tt> (decimal value 7) (as opposed to the normal <tt>PT_LOAD</tt>).

The virtual address of the thread local storage master segment is meaningless as it isn't loaded anywhere specific, you decide where you wish to load it. Mind that the segment does have alignment constraints like normal segments (but the linker placed those for you). Besides deciding yourself where to load the segment, you load this segment like a normal <tt>PT_LOAD</tt> segment.

=== Per-thread allocation ===

Each thread has a memory allocation associated with it. It contains the user-space thread structure, the thread local storage, and potentially other things. Each thread has a thread-self-pointer register that points to the thread's user-space thread structure, which is used to quickly determine which thread the current is, as well as providing quick access to the thread local storage. The exact semantics of the per-thread depends on the architecture and its ABI, as well as whether the executable is statically or dynamically linked.

The layout of the user-space thread structure is partially mandated by the ABI. On some platforms (i386 and x86-64), there must be a pointer at the start of it that points to itself. Besides those mandatory parts, the rest of the structure is up to you, it is useful for many things such as remembering allocations that must be deallocated when the thread terminates.

The thread local storage is located at a fixed offset from the user-space thread structure, therefore each variable in the thread local storage is also at a fixed offset. This offset is determined at link time and accessible through the special <tt>foo@tpoff</tt> linker symbols. Locating a particular thread local variable is as simple as getting the thread-self-pointer and adding a fixed offset.

== ABI ==

This is a summary of the actual details in the [[System V ABI]].

=== i386 ===

The thread-self-pointer register is the base of the %gs segment. It is set to the address of the current thread's user-space thread structure. When you switch the thread on the current CPU, you change the base of the gs segment of that CPU's [[GDT]] and reload the gs register. A pointer to the user-space thread structure itself is the first member of the user-space thread structure.

The thread local storage (after having its size rounded up to its alignment) is located immediately prior to the user-space thread structure. The offsets are negative. To place the user-space thread structure and the thread local storage, do this:

<source lang="c">
size_t allocation_alignment = max(master_tls_alignment, alignof(struct uthread));
size_t allocation_size = alignup(master_tls_size, allocation_alignment) + sizeof(struct uthread);
unsigned char* allocation = allocate(allocation_size, allocation_alignment);
struct uthread* uthread = allocation + alignup(master_tls_size, allocation_alignment);
unsigned char* tls = ((unsigned char*) uthread) - alignup(master_tls_size, master_tls_alignment);
</source>

Do note that the thread local structure might not be at the beginning of the per-thread allocation if it its alignment is less of that than struct uthread. It is crucial that both the thread local storage and the user-space thread structure are properly aligned. You then initialize the user-space thread structure's self pointer and the thread local storage:

<source lang="c">
uthread->self_pointer = uthread;
memcpy(tls, master_tls, master_tls_size);
</source>

=== x86-64 ===

The thread-self-pointer register is the base of the %fs segment. It is set to the address of the current thread's user-space thread structure. When you switch thread, you set the FSBASE [[MSR]] (0xC0000100). A pointer to the user-space thread structure itself is the first member of the user-space thread structure.

The per-thread allocation is arranged and placed as described under [[#i386]].

=== Other ===

See the <tt>tls.pdf</tt> document below and please document the specifics here afterwards. :)

== Implementation ==

Your kernel should bootstrap the thread local storage for the main thread after having loaded a program:

# During program load, locate the thread local storage segment among the program headers, and load it somewhere.
# Create the per-thread allocation for the main thread.
# Copy the master thread local storage to the thread local storage of main thread.
# In the main thread's user-space thread structure, store: The location/size of the per-thread allocation, master thread local storage location/size/alignment, main thread's thread local storage location/size, the main thread's stack location/size, and so on. This allows the thread to make new threads and to clean up after itself.
# Set the thread-self-pointer register of the main thread to the main thread's user-space thread structure.

This approach allows thread local storage to be operational immediately when a program is loaded and makes it simple to create new thread.

Setting up thread local storage for a new thread is simple:
# Create a per-thread allocation for the new thread.
# Copy the master thread local storage to the thread local storage of new thread.
# Initialize the user-space thread structure for the new thread.
# Set the thread-self-pointer register of the new thread to the new thread's user-space thread structure.

Some Unix kernels such as Linux actually doesn't set up the thread local storage for the main thread. The libc is required to parse the ELF executable of the program to locate and load the master thread local storage copy itself and bootstrap the main thread. This has the obvious disadvantages of having early times where language features doesn't work and that every executable gets linked in an ELF loader (in case it uses thread local storage).

== See Also ==

=== Standards ===

* [http://www.akkadia.org/drepper/tls.pdf ELF Handling For Thread-Local Storage]

=== Articles ===

* [[ELF]]
* [[GDT]]
* [[MSR]]
* [[System V ABI]]

=== Topics ===

* [http://forum.osdev.org/viewtopic.php?f=15&t=28339 Minimal Static Link] - [[User:Sortie|sortie]] posts about reducing startup libc bloat such as thread local storage initialization by moving it into the kernel to much success.

=== IRC ===

* [http://bespin.org/~qz/search/?view=1&c=osdev&y=14&m=11&d=17 #osdev 2014-11-17] - <tt>sortie</tt> and <tt>maxdev</tt> have a conversation about thread local storage.

=== Implementations ===

* [[User:Sortie/Sortix_Thread_Local_Storage|Sortix Thread Local Storage Implementation]] - Notes on [[User:Sortie|sortie]]'s implementation.

[[Category:ABI]]
[[Category:C]]
[[Category:Processes and Threads]]
````

<a id="w003674"></a>

## W003674 — User:Columbus/OR1K BareBones

Bron-ID: W003674
Namespace: User (2)
Revisie: 17079 | Datum: 2014-11-18T18:36:01Z
Bron: https://wiki.osdev.org/User:Columbus/OR1K_BareBones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AColumbus%2FOR1K_BareBones&oldid=17079

````text
Here a conversion of the [[Bare Bones]] tutorial to or1k.

== Sources ==
=== boot.asm ===
<pre>
# Declare constants for multiboot header.
.set ALIGN,    1<<0
.set MEMINFO,  1<<1
.set FLAGS,    ALIGN | MEMINFO
.set MAGIC,    0x1BADB002
.set CHECKSUM, -(MAGIC + FLAGS)

.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

.section .bootstrap_stack, "aw", @nobits
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

.section .text
.global _start
.type _start, @function
_start:
    # I think/hope, this sets the stack pointer
    l.movhi r3, hi(stack_top)
    l.ori   r3, r3, lo(stack_top)
    l.sw    0(r3), r1

    # I hope this calls kernel_main
    l.jal kernel_main

.Lhang:
    l.nop
    l.j .Lhang

.size _start, . - _start
</pre>

=== main.c ===
Can be the same, as in [[Bare Bones]].

=== linker.ld ===
Can be the same, as in [[Bare Bones]].

== Compilation ==
<source lang="bash">
or1k-elf-as boot.asm -o boot.o
or1k-elf-g++ -c main.c -o main.o -ffreestanding -O2 -Wall -Wextra -fno-exceptions -fno-rtti
or1k-elf-gcc -T linker.ld -o kernel -ffreestanding -O2 -nostdlib boot.o main.o -lgcc
</source>

== Execution ==
Honestly, I don't know, if anything was right about the compilation, or boot.asm, and I also have no idea how to execute it. On my machine everything worked fine, until the execution.
````

<a id="w003676"></a>

## W003676 — User:Phillid

Bron-ID: W003676
Namespace: User (2)
Revisie: 17088 | Datum: 2014-11-22T00:03:29Z
Bron: https://wiki.osdev.org/User:Phillid
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3APhillid&oldid=17088

````text
== Introduction ==
This page is a summary of successful builds of a [[GCC Cross-Compiler]] by members of OSDev.org. The numbers refer to the versions being built, not the host compiler doing the build.

== GCC Version 4.x.x ==

<!-- Excuse the hackish styling, I will request an edit of MediaWiki:Common.css -->
{| {{wikitable}}
|- style="height: 6em;"
! &darr; [[GCC]] / [[Binutils]] &rarr;
! style="transform: rotate(-90deg);max-width: 2em;" | 2.24 
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.2
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.23
! style="transform: rotate(-90deg);max-width: 2em;" | 2.22
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21.1a
! style="transform: rotate(-90deg);max-width: 2em;" | 2.21
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.20
! style="transform: rotate(-90deg);max-width: 2em;" | 2.19.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.19
! style="transform: rotate(-90deg);max-width: 2em;" | 2.18
! style="transform: rotate(-90deg);max-width: 2em;" | 2.17
! style="transform: rotate(-90deg);max-width: 2em;" | 2.16.1
! style="transform: rotate(-90deg);max-width: 2em;" | 2.16
|-
! 4.9.2
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.9.1
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.2
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.1
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.8.0
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.7.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.6.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.5.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.5.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.5.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.4.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.3.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
|-
! 4.3.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.3.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.3.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.4
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.2.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.1.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 4.1.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
|-
! 4.0.3
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
|-
! 4.0.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
|-
! 4.0.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
|-
! 4.0.0
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
|-
|}


* If you plan to use GCC 4.3.0 or a later version, you will also have to install the following (using your system's package management):
** [http://gmplib.org/ GNU GMP] (libgmp-devel on Cygwin, libgmp3-dev on apt-based systems, dev-libs/gmp on Gentoo)
*** [http://cs.nyu.edu/exact/core/gmp/ Tutorial] covering MSYS and MinGW builds of GMP
** [http://www.mpfr.org/ GNU MPFR] (libmpfr-devel on Cygwin, libmpfr-dev on apt-based systems, dev-libs/mpfr on Gentoo)

* If you plan to use GCC 4.5.0 or a later version, you will also have to install the following (using your system's package management):
** [http://www.multiprecision.org/ MPC] (libmpc-devel on Cygwin, libmpc-dev on apt-based systems, dev-libs/mpc on Gentoo)
* If you plan to use GCC 4.5.0 or a later version, and you would like to build libstdc++, you need to compile with a 4.5.x series compiler. Depending on your configuration, you may also need the --disable-libstdcxx-pch configure option.

== GCC Version 3.x.x ==
If for some reason, you need a GCC 3 build, these combinations have been tested. Note that if your code does not work with GCC 4, you usually have bigger problems.

{| {{wikitable}}
|-
! &darr; [[Binutils]] / [[GCC]] &rarr;
! 3.4.6
! 3.4.5
! 3.4.4
! 3.4.3
|-
! 2.23.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.23.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.23
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.22
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.21.1(a)
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.21
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.20.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.20
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.19.1
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.19
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
|-
! 2.18
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
|-
! 2.17  	 
| {{Yes}}
| {{Maybe|?}}
| {{Yes}}
| {{Maybe|?}}
|-
! 2.16.1  	 
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! 2.16
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|}

There have been successful builds with older versions, but many have (serious) problems as well. Mention of their specifics has been removed to keep this page short; please use up-to-date software.

== GCC on Mac OS ==
{| {{wikitable}}
|-
! &darr; [[Binutils]] / [[GCC]] &rarr;
! 4.6.2
! 4.6.1
! 4.5.2
! 4.5.1
! 4.5.0
! 4.4.2
! 4.4.0
! 4.2.4
|- 
! 2.23.2
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|- 
! 2.23.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|- 
! 2.23
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|- 
! 2.22
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|- 
! 2.21.1(a)
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|- 
! 2.21
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.20.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.20
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
| {{Maybe|?}}
|-
! 2.19.1
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
| {{Yes}}
| {{Maybe|?}}
|-
! 2.18
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Maybe|?}}
| {{Yes}}
|}

When compiling GCC 4.3 or higher on OS X 10.4 and 10.5, you may get unresolved symbol errors related to libiconv. This is because the version shipped with OS X is seriously out of date. Install a new version (compile it yourself or use macports) and add --with-libiconv-prefix=/opt/local (or /usr/local if you compiled it yourself) to GCC's ./configure line. Alternatively you may place the libiconv source in gcc-x.y.z/libiconv and it will be compiled as part of the GCC compilation process. (This trick also works for mpfr and gmp).

== See Also ==
=== Articles ===
*[[GCC Cross-Compiler]] - the tutorial that the configuration tables relate to.

[[Category:Compilers]]
````

<a id="w003677"></a>

## W003677 — User:Kazinsal

Bron-ID: W003677
Namespace: User (2)
Revisie: 22193 | Datum: 2018-03-09T07:24:06Z
Bron: https://wiki.osdev.org/User:Kazinsal
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKazinsal&oldid=22193

````text
Hi there. I'm Troy. I'm a system administrator and network engineer from Victoria, Canada. I am the lead developer of the Araxes operating system project. I am also a contributor to the Sortix operating system. I can be found in #osdev on Freenode.

I am a proponent of strong documentation and designing from a user experience perspective. There is no greater sin in operating systems development than having enough of a chain of failures from user experience downwards that the user or system administrator needs to look at the code to figure out how to use a tool or configure a feature.
````

<a id="w003678"></a>

## W003678 — User:Kazinsal/VirtualBox Video Adapter

Bron-ID: W003678
Namespace: User (2)
Revisie: 17254 | Datum: 2014-12-03T06:37:31Z
Bron: https://wiki.osdev.org/User:Kazinsal/VirtualBox_Video_Adapter
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKazinsal%2FVirtualBox_Video_Adapter&oldid=17254

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
[[image:Under_Construction.png|This page is under construction!]]
|
This page is a work in progress and may thus be incomplete. Its content may be changed in the near future.

<small>This page has notes attached to it: [[User:Kazinsal/VirtualBox Video Adapter/Notes]]</small>
|
|}
</center>

VirtualBox emulates a generic VESA compatible video device (internally referred to as '''VBoxVideo''') for video output, with a large superset of features built upon the [[BGA|Bochs Graphics Adapter]] and its VESA BIOS Extensions. For simple functionality, such as setting a video mode, the same sequence of commands can be used for VirtualBox's video adapter as is used for Bochs. VirtualBox supports up to and including 0xB0C4 level BGA VBE features for backwards compatibility with generic BGA drivers.

== Overview ==
As an extension of the Bochs Graphics Adapter, VBoxVideo uses a pair of I/O ports -- an index and a data port -- for most of its configuration and programming. These ports are at the same I/O addresses as in Bochs, VBE_DISPI_IOPORT_INDEX and VBE_DISPI_IOPORT_DATA, and serve the same purposes. In some cases, VBoxVideo extends the length of the registers to 32 bits; this is most notably used in the configuration of certain host-guest interactions.

The device supports irregular size displays, allowing you to set whatever resolution you want, up to 16384x16384x32, limited only by the amount of VRAM allocated to the virtual machine.

VBoxVideo is also a PCI device, with vendor 0x80EE (used for all VirtualBox PCI devices) and device 0xBEEF (sometimes 0x7145 in really old versions of VirtualBox).

== New #defines ==
There are three new levels of BGA versions reported by VirtualBox when reading VBE_DISPI_INDEX_ID:
* '''VBE_DISPI_ID_VBOX_VIDEO (<code>0xBE00</code>):''' Indicates that the graphics device supports VirtualBox Video Acceleration (VBVA).
* '''VBE_DISPI_ID_HGSMI (<code>0xBE01</code>):''' Indicates that the graphics device supports the VirtualBox Host-Guest Shared Memory Interface.
* '''VBE_DISPI_ID_ANYX (<code>0xBE02</code>):'''  Indicates that the graphics device supports any display width.

There are two new DISPI registers: 
* '''VBE_DISPI_INDEX_VBOX_VIDEO (<code>0x0A</code>):''' Used to read configuration information from and write commands to the host (sometimes).
* '''VBE_DISPI_INDEX_FB_BASE_HI (<code>0x0B</code>):''' Contains the high 16 bits of the linear framebuffer address (low 16 bits are zero).

== References ==
* [http://www.virtualbox.org/svn/vbox/trunk/src/VBox/Devices/Graphics/BIOS/vbe_display_api.txt VirtualBox SVN: Bochs VBEBios documentation (incomplete)]
* [http://www.virtualbox.org/svn/vbox/trunk/src/VBox/Devices/Graphics/DevVGA.cpp VirtualBox SVN: DevVGA.cpp]

[[Category:In Progress]]
````

<a id="w003679"></a>

## W003679 — User:Kazinsal/BlacklightEVO

Bron-ID: W003679
Namespace: User (2)
Revisie: 18673 | Datum: 2015-12-05T23:01:15Z
Bron: https://wiki.osdev.org/User:Kazinsal/BlacklightEVO
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKazinsal%2FBlacklightEVO&oldid=18673

````text
{{OSProject
|OSName=BlacklightEVO
|OSDescription=BlacklightEVO is an in-development free operating system for x86 PCs. The operating system is being designed with simplicity in mind, using a new design unlike (and incompatible with) traditional Unix-like systems. BlacklightEVO is available under the terms of the ISC license.
|Contact=Kazinsal
|URL=http://www.blacklightevo.org
|Status=active
}}

{{SmallNavBox
|width:275px;
|Name=BlacklightEVO
|Group0=http://i.imgur.com/htUZBHu.png
|List0=The BlacklightEVO project logo.<hr />
|Group1=General Information
|List1=
<table style="width: 100%">
<tr><td style="text-align: left; width: 40%;">'''Developer'''</td><td style="text-align: left;">The Cordilon Group</td></tr>
<tr><td style="text-align: left; width: 40%;">'''Source model'''</td><td style="text-align: left;">Open source ([https://opensource.org/licenses/ISC ISC])</td></tr>
<tr><td style="text-align: left; width: 40%;">'''Platforms'''</td><td style="text-align: left;">[[x86]] (IA-32)</td></tr>
<tr><td style="text-align: left; width: 40%;">'''Latest release'''</td><td style="text-align: left;">Unreleased</td></tr>
</table>
|Group2=Kernel Information
|List2=
<table style="width: 100%">
<tr><td style="text-align: left; width: 40%;">'''[[Languages]]'''</td><td style="text-align: left;">[[C]], [[Assembly]] (x86)</td></tr>
<tr><td style="text-align: left; width: 40%;">'''[[Kernel|Kernel model]]'''</td><td style="text-align: left;">Monolithic kernel</td></tr>
<tr><td style="text-align: left; width: 40%;">'''Toolchain'''</td><td style="text-align: left;">[[GCC]], [[NASM]]</td></tr>
<tr><td style="text-align: left; width: 40%;">'''POSIX'''</td><td style="text-align: left;">No (ELF, custom ABI)</td></tr>
<tr><td style="text-align: left; width: 40%;">'''Filesystems'''</td><td style="text-align: left;"><!--ext2, -->bootram</td></tr>
</table>
|Group3=Userspace Information
|List3=
<table style="width: 100%">
<tr><td style="text-align: left; width: 40%;">'''i18n'''</td><td style="text-align: left;">English</td></tr>
<!--<tr><td style="text-align: left; width: 40%;">'''Toolchain'''</td><td style="text-align: left;">tbd</td></tr>-->
</table>
|Group4=Other Information
|List4=
<table style="width: 100%">
<tr><td style="text-align: left; width: 40%;">'''Official website'''</td><td style="text-align: left;">[http://www.blacklightevo.org www.blacklightevo.org]</td></tr>
<tr><td style="text-align: left; width: 40%;">'''Lead developer'''</td><td style="text-align: left;">[[User:Kazinsal|Kazinsal]]</td></tr>
</table>
}}
````

<a id="w003680"></a>

## W003680 — Object oriented OS

Bron-ID: W003680
Namespace: Main (0)
Revisie: 30403 | Datum: 2026-03-24T22:07:04Z
Bron: https://wiki.osdev.org/Object_oriented_OS
Revisie-URL: https://wiki.osdev.org/index.php?title=Object_oriented_OS&oldid=30403

````text
{{Template:Kernel designs}}

I ''([[User:Columbus|Columbus]] 15:41, 28 November 2014 (CST))'' thought, that it would be a good idea to document the discussion and shorten down the thoughts that were made in it. '''I don't want to expand the discussion to the wiki.'''
Don't we play very often devil's advocate in the forums, when we come up with some very special idea? I want to document these ideas, so we help others to think out of the box, or to tell, why we are in the box we are now.

== Idea ==
=== XML ===
http://forum.osdev.org/viewtopic.php?f=15&t=27971&sid=09c18db1e75118b68f3469f3accdf8fc

The idea was to code using a Language in which the different statements were XML nodes.
It was also discussed, how an IDE for this "Language" would look like, or what it could do.
Some part of the discussion was about the way someone shows the code. In an assembly kind of way, or an C# or XML kind of way.
==== Pros ====
The first idea was, to use XML because it extremely structured/hierarchical.
==== Cons ====
Some people said it doesn't support functions so well. And leads to complicated function calls or variable declarations.
Also the code would get much more complicated that it would be in a more common language like [[C]] or [[Assembly]].
This was due to the "unusable" syntax. The tags must be closed. With a complete repeat of the tag at the beginning, while in other languages one can just write }, ] or ).

One post said:"Congratulations, you win the verbosity award!"

Another problem was, how to differentiate between different usages of assembly instructions like "mov eax, ebx", "mov eax, 32". There wasn't a good way to do that in the proposed XML-type Language.

Then there was XML itself, which was called as "worse for everything". Because of "worse readability and efficiency". ''But this is debatable.''

=== Everything is an Object ===
http://forum.osdev.org/viewtopic.php?f=15&t=27544&sid=8f693f1c17825da8f4087b6f4a9d6769

[[Category:Kernel]]
````

<a id="w003681"></a>

## W003681 — ARM RaspberryPi Tutorial C

Bron-ID: W003681
Namespace: Main (0)
Revisie: 17154 | Datum: 2014-11-30T22:44:49Z
Bron: https://wiki.osdev.org/ARM_RaspberryPi_Tutorial_C
Revisie-URL: https://wiki.osdev.org/index.php?title=ARM_RaspberryPi_Tutorial_C&oldid=17154
Doorverwijzing: Raspberry Pi Bare Bones

````text
#REDIRECT [[Raspberry Pi Bare Bones]]
````

<a id="w003682"></a>

## W003682 — Talk:ARM RaspberryPi Tutorial C

Bron-ID: W003682
Namespace: Talk (1)
Revisie: 17156 | Datum: 2014-11-30T22:44:50Z
Bron: https://wiki.osdev.org/Talk:ARM_RaspberryPi_Tutorial_C
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AARM_RaspberryPi_Tutorial_C&oldid=17156
Doorverwijzing: Talk:Raspberry Pi Bare Bones

````text
#REDIRECT [[Talk:Raspberry Pi Bare Bones]]
````

<a id="w003683"></a>

## W003683 — User:Combuster/Frankenstein's MSVC

Bron-ID: W003683
Namespace: User (2)
Revisie: 17335 | Datum: 2014-12-07T20:25:45Z
Bron: https://wiki.osdev.org/User:Combuster/Frankenstein%27s_MSVC
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FFrankenstein%27s_MSVC&oldid=17335

````text
{{Lovecraftian}}

Salvaged black magic rituals nobody seems to have tried in three years. Needs some cleanup; use at your own risk.

= Introduction =

Have you ever dreamt of using Microsoft C/C++ Optimizing Compiler on Linux? Did you have trouble programming in an unfamiliar GCC environment? This article is the answer to configuring the build environment that you dreamt of; it will solve any problems that you had with GCC!

= Configuration =
There are many ways that you can utilise Visual C++ (Microsoft C/C++ Optimizing Compiler) for OS development. Firstly, you could simply use Visual C++ on Windows with all Microsoft's tools. But, this poses a lot of problems. For example: Microsoft Linker (link.exe) does not support linker scripts. Since it may be critical that you can 'model' the final executable image, this may pose a problem. The available solutions are not too great in number:
# Use GCC and Binutils on Cygwin (don't use VC++ at all).
# Use Binutils with COFF support to link COFF object files created by '''cl.exe''' (ditch VC++ linker).

It's the second solution that is interesting. You may ask "why not try it the other way around?", but that would remove the benefit of having linker scripts, although it would offer you GCC's 'style'. If you were aiming for the latter, then it may be best for you to just build a [[GCC_Cross-Compiler|GCC cross-compiler]].

== Prerequisites ==
* Windows system - In order to legally do this, you must have a valid Windows licence (this is due to the 'winetricks' procedure).
** Visual Studio (C++) - You need to have Visual C++ installed on your Windows system in order to get the compilers from it. If you don't, you can download an Express version from the Microsoft website.

* Linux system (or any other compatible system)
** Basic build environment - used to build [[Binutils]].
** Wine - used to run Microsoft C/C++ Optimizing Compiler on Linux.

Last but not least, having an X Window system on your Linux box is recommended to make your life easier while doing the 'winetricks' procedure, but not required. You may also want to check the [http://appdb.winehq.org/ Wine Database] to see what versions of Wine run which versions of the compiler.

== Running Microsoft C/C++ Optimizing Compiler on Linux ==
# Go to your Visual C++ binary directory (ex: C:\Program Files (x86)\Microsoft Visual Studio 9.0\VC\bin).
# Copy all files into your Linux system (use FTP, SFTP, SMB or any equivalent file transfer methods). Tip: make sure to copy 1033 directory. 1033\clui.dll is used by cl.exe and must exist under (cl.exe Directory Path)\1033. If you are only willing to build i386 program, only copy the files in the binary directory and 1033 directory (you don't need to copy the files under x86_amd64, x86_ia64).
# Copy mspdb**.dll from your Visual Studio Common7 directory to Wine system32 directory (ex: C:\Program Files (x86)\Microsoft Visual Studio 9.0\Common7\IDE\mspdb80.dll to ~/.wine/drive_c/windows/system32).
# Try to run cl.exe and see if it works (ex: wine cl.exe). If it displays an error message (that is caused by a runtime DLL), do the following steps, otherwise skip to step 4.
## cd ~/.wine
## wget http://www.kegel.com/wine/winetricks (or any equivalent methods to download the specified file from the web.)
## sh winetricks corefonts vcrun6 vcrun2003 vcrun2005 vcrun2008 (install the runtime version that you need. see http://wiki.winehq.org/winetricks for more information.) Tip: make sure that you run this command under X-Window. Otherwise, you will not see the EULA user acceptance window.
## Go back to the directory where you saved the Microsoft C/C++ Optimizing Compiler executable files and try running cl.exe. It should work in this time. If it does not, make sure that you copied all required files from Visual Studio directory and 'winetricks' ran without any error messages.
# Enjoy

== Configuring Binutils for Microsoft C/C++ Optimizing Compiler ==
# Download the latest version of Binutils source code
# Unzip (or untar, unxz, whatever) the source code
# mkdir&cd (Binutils Directory Name)-(Major Target BFD) (In this example, we will use i386-elf)
# ../(Binutils Directory Name)/configure --target=i386-elf --enable-targets=i386-coff (see (Binutils Directory Name)/ld/configure.tgt for other architecture)
# make all (check any errors and fix them.)
# make install

= Building an OS image =
Now, if you've followed all steps with no errors, you can build your Hello World image!

----
Test.cpp
<source lang="cpp">
char *VideoMemory = (char *)0xB8000;
 
extern "C" void Main()
{
    const char *HelloWorldMessage = "Hello World!";

    while (*HelloWorldMessage != '\0')
        VideoMemory++ = HelloWorldMessage++;
}
</source>
----
TestLS.ld
 ENTRY(_Main)
 
 BASEADDRESS = 0x100000;
 
 SECTIONS
 {
     . = BASEADDRESS;
 
     .text : AT(ADDR(.text) - BASEADDRESS)
     {
         *(.text)
     }
 
     .data : AT(ADDR(.data) - BASEADDRESS)
     {
         *(.data)
     }
 }
----
# Register the Microsoft C/C++ Optimizing Compiler executable file path in the PATH environment variable.
# Give x permission to all executable files (will automatically be run by Wine).
# cl.exe /Gd /Zl /Od /GS- /X /FAcs /FaTest.lst /FoTest.obj /c Test.cpp
# i386-elf-ld --format coff-i386 --oformat binary -o Test.x -T TestLS.ld -nostdlib Test.obj (Tip: --oformat can be anything you want; as long as the binutils that you built supports it, for ex: elf32-i386).
# Write a loader that loads the image file in Protected Mode at 0x100000 and run it!

= FAQ =
=== I've never heard of the Microsoft C/C++ Optimizing Compiler. What is it? ===
That's not true. You've always been hearing about it. It's the compiler that the Visual C++ IDE uses as default (comes with the C++ component of Visual Studio).

=== Why use it on Linux when there is GCC to do the job? ===
It's mainly because of one's preference. GCC has its own style, and so does Microsoft C/C++ Optimizing Compiler. If you like it, you might want to use it on Linux as well. In addition, if you have been using it for years, it's not strange to have a preference over another.
````

<a id="w003684"></a>

## W003684 — User talk:Combuster/Frankenstein's MSVC

Bron-ID: W003684
Namespace: User talk (3)
Revisie: 17183 | Datum: 2014-12-01T20:45:24Z
Bron: https://wiki.osdev.org/User_talk:Combuster/Frankenstein%27s_MSVC
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3ACombuster%2FFrankenstein%27s_MSVC&oldid=17183

````text
== A couple of notes ==
# In "Building an OS image", the file Test.cpp AFAIK does an invalid write to video memory; the color attribute byte used in video memory (of which each cell is a '''word''') is overwritten with the next character of the word. You may want to fix that to set a standard color.
# I think it may be handy if you also posted the versions of Wine and Visual C++ you used with this article. AFAIK not all versions of Wine work with each version of the VC++ compiler equally well.

[[User:Creature|Creature]] 09:57, 30 July 2010 (UTC)

== Optimizing Compiler? ==

I don't like this article as it indirectly suggests that all other compilers are *not* optimizing. Additionally, "Microsoft C/C++ Optimizing Compiler" is *not* the name of the compiler, though I forgot what the official name is. "Microsoft C Compiler", "Visual Studio", "Visual C Compiler"? Anyways, probably a good idea to rename this article. --[[User:Sortie|Sortie]] 13:14, 4 November 2013 (CST)
:Oh god. I deleted this article once and it didn't take! --[[User:Sortie|Sortie]] 14:45, 1 December 2014 (CST)
````

<a id="w003685"></a>

## W003685 — Microsoft C/C++ Optimizing Compiler Build Environment on Linux

Bron-ID: W003685
Namespace: Main (0)
Revisie: 17176 | Datum: 2014-12-01T19:32:25Z
Bron: https://wiki.osdev.org/Microsoft_C/C%2B%2B_Optimizing_Compiler_Build_Environment_on_Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=Microsoft_C%2FC%2B%2B_Optimizing_Compiler_Build_Environment_on_Linux&oldid=17176
Doorverwijzing: User:Combuster/Frankenstein's MSVC

````text
#REDIRECT [[User:Combuster/Frankenstein's MSVC]]
````

<a id="w003686"></a>

## W003686 — Talk:Microsoft C/C++ Optimizing Compiler Build Environment on Linux

Bron-ID: W003686
Namespace: Talk (1)
Revisie: 17178 | Datum: 2014-12-01T19:32:25Z
Bron: https://wiki.osdev.org/Talk:Microsoft_C/C%2B%2B_Optimizing_Compiler_Build_Environment_on_Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMicrosoft_C%2FC%2B%2B_Optimizing_Compiler_Build_Environment_on_Linux&oldid=17178
Doorverwijzing: User talk:Combuster/Frankenstein's MSVC

````text
#REDIRECT [[User talk:Combuster/Frankenstein's MSVC]]
````

<a id="w003688"></a>

## W003688 — My Bootloader Does Not Work

Bron-ID: W003688
Namespace: Main (0)
Revisie: 27998 | Datum: 2023-07-09T17:42:22Z
Bron: https://wiki.osdev.org/My_Bootloader_Does_Not_Work
Revisie-URL: https://wiki.osdev.org/index.php?title=My_Bootloader_Does_Not_Work&oldid=27998

````text
Here follows a list of the (excessively made beginner) mistakes that occur when writing a bootsector. There are several tutorials out there that suggest really broken behaviour, and there are many errors typically coming out of ignorance. 

If you came here wondering how to start at all, have a look at [[Rolling Your Own Bootloader|rolling your own]] or [[bootloader]]s in general.

[[GAS]]-specific issues will not be discussed here: it is the only common assembler that can not emit flat binaries, and as such requires the aid of a significantly more complicated linker. [[YASM]] supports GAS syntax without adding the huge problem surface it brings. Particularly beginners will want to switch until they have understood the problems they're having.

== Draw a [[Memory Map (x86)|map]] ==
Or type it out. An x86 bootsector is loaded at 0x00007C00 in physical memory. There's a BIOS area at 0x00000000 to 0x000004FF, and there's an EBDA somewhere between 0x00080000 and 0x0009FFFF. There's a lot of memory again after 0x00100000 but you can't tell the BIOS to use it, you'll have to check for it, enable it, and real mode dislikes it in general, so you don't want to avoid going there as long as possible.

Plot in this map where you will put a stack (give it 0x1000 bytes), where you want to load any pieces from the disk like second stages, [[FAT]] tables and anything else that does not come preallocated within your bootsector. Look up how big everything is, and make sure nothing overlaps with anything else. You need this later, and 

'''You will have to tell us what's in your map if you decide to ask a question later'''

=== You read something from disk and did not put it on the memory map ===
And more often than not, the fact that you're looking here is that you just loaded a sector from disk over your stack or your existing code.

== 90% of the errors happen within the first 10 lines of assembly ==
And more than often several. Fix them all.

In real mode, memory is always accessed as segment*16 + offset. There's always a segment involved, and there's always an offset involved in anything real-mode related. Now that you have your map, you'll have to find yourself pairs of segments and offsets that point to each of those addresses. There are three standard methods of going at it.
* '''The segment is zero, making the offset the physical address.''' This limits you to the first 64k of memory, but that might also exactly be the memory you want.
* '''The offset is zero, making the segment the physical address divided by 16.''' You're not allowed to round off. You also need to be careful with that you need to set segment registers all over the place.
* '''The offset is the lowest 16 bits of the address, and the next four bits end up in the segment followed by three zeroes'''. Write it out on paper. It's more complicated, but it works for any address in the 640K you could think of.

=== There's no ORG statement ===
Your code involves a CS and an IP, the segment and offset. Your data is in the same block, using another segment register and offset. ORG tells the assembler at which offset, not the segment, you will be using it. If you picked segment=0 and offset=0x7C00, then ORG 0x7C00 is correct. If you pick segment=0x07C0 and offset=0 then ORG 0 is required. This might happen to be the default for many assemblers, but if you don't put it there we will assume it's wrong anyway.

=== You are assuming a wrong CS:IP value ===
Some BIOS start the bootloader with the CS 0x7C0 and IP 0. Others with CS 0 and IP 0x7C00. Both is the same memory location but different segment registers.
Do a far JMP to set it to your preferred value. For example:
 ORG 0x7C00
 start: JMP 0:skip
 skip: ... your code here ...

=== There's no BITS 16 statement ===
There are several execution modes. Don't let your assembler guess one.

=== There's no reference to SS or SP ===
All, but a select few of the registers are undefined. The stack is not where you put it on your map. Set it or things will break.

 MOV AX, 0
 MOV SS, AX
 MOV SP, 0x7C00

This puts the stack directly in front of your bootsector. That means you also occupy 0x6C00-0x7BFF and can't use it for anything else after this.

=== The value of SP ends in an 0xF ===
You stole some code off the internet. Shame on you, and I hope you learnt not to use tutorials in the future. The stack should be a multiple of two in real mode (the number of bytes you PUSH and POP each time). Failure to do so makes your code slower - up to 25%!

=== The value of SP points to the beginning of the stack in memory ===
The stack grows downwards. SP should be the first byte after the stack's memory. If that would make it 0x10000, set it to zero, because in 16-bit numbers the -2 it will do before writing will automatically wrap it around back to 0xFFFE

=== The value of SP ends in an 0xE ===
SP is reduced by two for each push operation. Suddenly it writes two bytes at 0x..C and 0x..D wasting the bytes at 0x..E and 0x..F

=== There's a PUSH, CALL or INT before SS is being set ===
You just wrote over some memory SS:SP happened to point at. Now every BIOS call will fail because of this.

=== SP isn't set in the instruction immediately after SS ===
If an interrupt happens, it will use an old SP and a new SS. Setting SS prevents interrupts for the next instruction, so that has to be SP.

=== There's no reference to DS or ES ===
You can't use any data in your bootsector. Every piece of data is accessed by a segment and an offset, so you have to set the segment because it's not pointing anywhere yet. The assembler will deal with the offsets if you have the right ORG statement.

Several BIOS calls require you to supply a value in a segment register. Be sure to restore it again.

=== There's a save of the boot drive number before DS is set ===
To reiterate on the previous point, you cannot e.g. store the [[MBR (x86)#MBR_Bootstrap|boot drive number]] (passed to your bootloader in the DL register) in a variable at some memory location before you've first correctly set up DS.

=== A segment register contains 9xxx or 8xxx ===
You're writing into the EBDA and you just broke the BIOS.

== BIOS specific problems == 
Unfortunately, the BIOS is poorly standardized{{how}} leading to various and sometimes poorly written implementations. Because of this BIOS specific situations can arise that will cause it not to load a bootloader. As an example some BIOSes may require a valid partition table to exist at byte address 0x1BE of the storage device. This is very common in the compatibility mode of UEFI machines, as the reference implementation attempts to parse the partition table before branching between the compatibility and UEFI modes. Furthermore, if you use older hardware for testing, the BIOS may not support INT 13h extensions - most notably LBA support.

[[Category:Troubleshooting]]
[[Category:FAQ]]
````

<a id="w003689"></a>

## W003689 — Template:Lovecraftian

Bron-ID: W003689
Namespace: Template (10)
Revisie: 17231 | Datum: 2014-12-02T22:26:32Z
Bron: https://wiki.osdev.org/Template:Lovecraftian
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ALovecraftian&oldid=17231

````text
<center>
{|style="border: 1px solid #8f9f8f; padding: .0em .25em .0em; background-color: #a0aaa0; text-align: center;"
|
http://i.imgur.com/4oMm5jg.png
|
'''Ph'nglui mglw'nafh Cthulhu R'lyeh wgah'nagl fhtagn!'''

<font color="black">The content of this article or section is [[:Category:Lovecraftian|Lovecraftian]].</font><br />
|
|}
<includeonly>[[Category:Lovecraftian]]</includeonly>
</center>
<br />
````

<a id="w003690"></a>

## W003690 — Category:Lovecraftian

Bron-ID: W003690
Namespace: Category (14)
Revisie: 17259 | Datum: 2014-12-03T18:03:35Z
Bron: https://wiki.osdev.org/Category:Lovecraftian
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALovecraftian&oldid=17259

````text
{{Lovecraftian}}
Lovecraftian horror is a sub-genre of operating systems development that emphasizes the cosmic horror of the unknown (and in some cases, unknowable) more than gore or other elements of shock, though these may still be present. It is named after American author H. P. Lovecraft (1890–1937).
````

<a id="w003691"></a>

## W003691 — User:Kazinsal/VirtualBox Video Adapter/Notes

Bron-ID: W003691
Namespace: User (2)
Revisie: 17255 | Datum: 2014-12-03T06:46:00Z
Bron: https://wiki.osdev.org/User:Kazinsal/VirtualBox_Video_Adapter/Notes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKazinsal%2FVirtualBox_Video_Adapter%2FNotes&oldid=17255

````text
'''Readers: Do not take these notes as fact. These are my musings and such attributable to my downward spiral into insanity while working on the VBoxVideo research. Proceed at your own risk.'''

* [http://www.virtualbox.org/svn/vbox/trunk/src/VBox/Additions/x11/vboxvideo/ The X11 sample driver is shit.] Seriously, this thing is a nightmare. I've spent about an hour trying to figure out how it works, and while it seems simple compared to the other, less generic drivers, it's godawful at best.
** ''Reminder to self:'' Increment count of hours wasted reading this helldump of code whenever notes change.
* There is no good explanation anywhere that I can find as to how HGSMI actually works.
* "Size hints" seem to be how a guest's VBoxVideo driver is supposed to know when to change the resolution.
** ''Conjecture:'' Presumably size hints are only sent when VBE_DISPI_ID_ANYX is available, as other versions of the graphics device must have <code>(cx % 8 == 0)</code>.
````

<a id="w003692"></a>

## W003692 — User:Schol-r-lea

Bron-ID: W003692
Namespace: User (2)
Revisie: 21075 | Datum: 2017-08-10T13:15:18Z
Bron: https://wiki.osdev.org/User:Schol-r-lea
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASchol-r-lea&oldid=21075

````text
I have... returned.

I am finally getting somewhere with my work, even if I am mostly focusing on the toolchain at the moment, and I have Big Plans for the eventual implementation of my new language, [http://www.thelema-language.org Thelema], my assembler, [https://github.com/Schol-R-LEA/Assiah Assiah], and my operating system, Kether. 

Oh, and regarding the .sig: Yes, it's true, Lisp and Scheme programmers often - indeed, in my experience almost always - get at least a glimmer of that [http://xkcd.com/224/ glorious Illumination (fnord) experience] that is so hard to explain to those who haven't had it themselves, this beautiful vision of '''understanding''' and seeing it all make sense and just seem so ''right''... an experience that inevitably is spoiled when you come back down to earth and realize that the elegance you just envisioned is almost impossible in real life. Ah, well.

Lispers are not the only ones who get this - Smalltalk and Forth programmers get it fairly often too, and some assembly programmers seem to as well - but it seems to be most common with Lisp (if you pardon the pun). It really is difficult to express how simply perfect it seems to those who haven't had the same experience, and even those who do tend to come away from it with different answers as to what it means. I guess it really is a religious experience, in a sense.

I may be setting OS-dev aside for a while (again) due to other projects which take precedence. We'll see how that goes.
````

<a id="w003693"></a>

## W003693 — User:Sortie/Creating an Operating System

Bron-ID: W003693
Namespace: User (2)
Revisie: 17272 | Datum: 2014-12-03T20:54:04Z
Bron: https://wiki.osdev.org/User:Sortie/Creating_an_Operating_System
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie%2FCreating_an_Operating_System&oldid=17272
Doorverwijzing: Creating an Operating System

````text
#REDIRECT [[Creating an Operating System]]
````

<a id="w003694"></a>

## W003694 — User:MessiahAndrw/LLVM OS Specific Toolchain

Bron-ID: W003694
Namespace: User (2)
Revisie: 25731 | Datum: 2021-03-04T19:52:17Z
Bron: https://wiki.osdev.org/User:MessiahAndrw/LLVM_OS_Specific_Toolchain
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMessiahAndrw%2FLLVM_OS_Specific_Toolchain&oldid=25731

````text
These instructions are on building an [[OS Specific Toolchain]] using [http://llvm.org LLVM] and [http://clang.llvm.org clang] instead of [[GCC]].

'''THIS IS A WORK IN PROGRESS''' - DON'T FOLLOW THESE YET!

Note that LLVM by default builds cross-compilers for all targets, the right target simply have to be activated (why is there a [[LLVM Cross-Compiler]] page?) These instructions add your operating system as a new target. By following these instructions, hopefully you will have a LLVM-based toolchain for building applications that run under your OS!

Note that LLVM is developed in C++11 and uses many of the language's modern features, and there have been reported difficulties compiling some components on other compilers (such as GCC). If you run into trouble, you can try compiling LLVM with Clang.

== Tools ==
These are the tools (built on [http://llvm.org/ LLVM]) that we will be building and using:
* [http://clang.llvm.org/ clang] - A C/C++/Objective-C compiler frontend.
* [http://libcxx.llvm.org/ libc++] - A standard C++ library with all of the C++11 bells and whistles.
* [http://libcxxabi.llvm.org/ libc++abi] - The portable ABI behind libc++.
* [http://lld.llvm.org/ lld] - A linker.
* [http://llvm.org/docs/CommandGuide/llvm-as.html LLVM-as] - An assembler that comes with LLVM.
* [https://llvm.org/devmtg/2016-01/slides/Porting_LLVM_to_a_new_OS.pdf LLVM Developers' Meeting] -  Presentation about porting llvm to new os.(2016 year)
* [https://github.com/osquery/osquery-toolchain osquery-toolchain] -  Script in repository is used to build the LLVM/Clang toolchain which is used in the osquery project to create portable binaries of it.

== Examples == 
* [https://lf-net.org/blog/posts/2020-05-17-clang-osdev Porting llvm+clang] - Artical about porting llvm to new os
* [https://praios.lf-net.org/littlefox/llvm-project/-/compare/500864f928c272e8ebfd6493cb749083124bfd8b...lf-os llvm LF OS changes] - Git diff for small support new os
* [https://github.com/lexasub/llvm-project-kos Kolibri OS LLVM] - Example porting llvm+clang+lld to Kolibri OS(IN DEVELOPMENT)

== Checking out ==
First we must check out the source code and create the basic structure. We'll assume we want two directories - a 'llvm' directory containing the source code, and 'build' containing our build toolchain.

Check out the source LLVM+Clang source code:
 git clone https://github.com/llvm/llvm-project.git
Make a build directory
 mkdir llvm-project/build
 cd llvm-project/build


==Cmake files==
<b>NEED REWRITE TO CMAKE</b>

===[TOOLCHAIN??]Add you os to clang/lib/Driver/CMakeLists.txt===
  ...
  ToolChains/Hexagon.cpp
  ToolChains/Hurd.cpp
  ToolChains/Linux.cpp
  ToolChains/Myos.cpp #add this
  ToolChains/MipsLinux.cpp
  ToolChains/MinGW.cpp
  ToolChains/Minix.cpp
  ...

===CUSTOM LINKER===
====Add you os to lld/tools/lld/CMakeLists.txt====
  ...
  lldMachO2
  lldMinGW
  lldWasm
  lldMYOS
  ...
  )
  ...
====Add you os to lld/CMakeLists.txt====
  ...
  add_subdirectory(MachO)
  add_subdirectory(MinGW)
  add_subdirectory(wasm)
  add_subdirectory(MYOS)
  ...

== Modifying LLVM ==
The first step is to get LLVM to recognize your OS as a platform. Like the [[OS Specific Toolchain|other tutorial]], we'll assume your OS is called MyOS. Of course, you'd replace MyOS with your own operating system's name (unless your OS is called MyOS).
===llvm/include/llvm/ADT/Triple.h===
In the enum called OSType (around line 120), add your OS:
<source lang="cpp">
 enum OSType {
   ...
   MyOS,
   ...
 };
</source>

===llvm/lib/Support/Triple.cpp===
In the function Triple::getOSTypeName (around line 135) add your OS:
  case MyOS: return "myos";

In the function parseOS (around line 326), add your OS:
  .StartsWith("myos", Triple::MyOS)

Around line 414 is getDefaultFormat that returns the default executable format type for a platform. The fallback is ELF, but if you want to use PE or MachO (or maybe your own) you can stick it here.

===llvm/lib/Support/* Notes===
There is some platform specific stuff in llvm/lib/Support/* (particularly Hosts.cpp and the subdirectories), but they appear to be support files for the platform the compiler runs on, not targets.

==Modifying LLD==
===lld/tools/lld/lld.cpp===
    ...
    .CasesLower("wasm", "ld-wasm", Wasm)
    .CasesLower("myos", "ld-myos", MyOS)
    .CaseLower("link", WinLink)
    ...
    case Wasm:
      return !lld::wasm::link(args, exitEarly, stdoutOS, stderrOS);
    case Myos:
      return !myos::link(args, exitEarly, stdoutOS, stderrOS);
    ...
    die("lld is a generic driver.\n"
        "Invoke ld.lld (Unix), ld64.lld (macOS), lld-link (Windows), wasm-ld"
        "Invoke ld.lld (Unix), ld64.lld (macOS), lld-link (Windows), wasm-ld"
        " (WebAssembly) instead");
        " (WebAssembly), myos-ld (MyOS) instead");
    ...
===lld/include/lld/Common/Driver.h===
   ...
   namespace myos {
     bool link(llvm::ArrayRef<const char *> args, bool canExitEarly,
          llvm::raw_ostream &stdoutOS, llvm::raw_ostream &stderrOS);
   }
   ...

<b>ADD EXAMPLE LLD KOLIBRI</b>

If your operating system uses its own executable format, you can find the relevant code under llvm/projects/lld/lib/ReaderWriter/, but this is a much more difficult job than porting using a common executable format like ELF, MachO, or PE.

'''NOTE: I am a different contributor (not MessiahAndrw), and my OS does use a different executable format. I will document my progress on that later.'''

==Modifying Clang==
===clang/lib/Basic/Targets/OSTargets.h===
We need to create a target so Clang knows a little bit about the platform it's compiling for, so we will create a TargetInfo object called MyOSTargetInfo. You can override some compiler internals here (such as setting the size of long ints) - look at what the other targets do for example.

Somewhere in this file, above AllocateTarget, create your target object:
<source lang="cpp">
... //other targets
 // MyOS target
template<typename Target>
class LLVM_LIBRARY_VISIBILITY MyOSTargetInfo : public OSTargetInfo<Target> {
 protected:
  void getOSDefines(const LangOptions &Opts, const llvm::Triple &Triple,
                    MacroBuilder &Builder) const override {
    Builder.defineMacro("_MYOS");
  }

 public:
   MyOSTargetInfo(const llvm::Triple &Triple)
       : OSTargetInfo<Target>(Triple) {
     this->WIntType = TargetInfo::UnsignedInt;
     switch (Triple.getArch()) {
      default:
       break;
        /*case llvm::Triple::mips:
        case llvm::Triple::mipsel:
        case llvm::Triple::mips64:
        case llvm::Triple::mips64el:
        case llvm::Triple::ppc:
        case llvm::Triple::ppcle:
        case llvm::Triple::ppc64:
        case llvm::Triple::ppc64le:
        this->MCountName = "_mcount";
          break;*/
        case llvm::Triple::x86:
        case llvm::Triple::x86_64:
         this->HasFloat128 = true;
         break;
     }
   }
   const char *getStaticInitSectionSpecifier() const override {
    return ".text.startup";
   }
};
... //other targets
</source>

===clang/lib/Basic/Targets.cpp===
In AllocateTarget, you'll need to add your OS in switch(Triple.getArch()):
<source lang="cpp">
switch (Triple.getArch()) {
   ...
   case llvm::Triple::x86: // and/or llvm::Triple::x86_64
      ...
      switch (os) {
         ...

        case llvm::Triple::MyOS:
            return new MyOSTargetInfo<X86TargetInfo>(Triple,Opts); // or MyOSTargetInfo<X86_64TargetInfo>
         ...
    ...
 }
</source>

===clang/lib/Driver/ToolChains/Myos.h===
Example: https://raw.githubusercontent.com/lexasub/llvm-project-kos/main/clang/lib/Driver/ToolChains/Kolibri.h

Next, we have to create a toolchain object that Clang uses to figure out how to connect to the other toolchain components (namely the linker and assembler) for our target.

Add this somewhere:
<source lang="cpp">
 class LLVM_LIBRARY_VISIBILITY MyOS : public Generic_ELF {
 public:
   MyOS(const Driver &D, const llvm::Triple &Triple,
        const llvm::opt::ArgList &Args);

    /*bool HasNativeLLVMSupport() const override;

    void
    AddClangSystemIncludeArgs(const llvm::opt::ArgList &DriverArgs,
                              llvm::opt::ArgStringList &CC1Args) const override;
    void addLibStdCxxIncludePaths(
        const llvm::opt::ArgList &DriverArgs,
        llvm::opt::ArgStringList &CC1Args) const override;
    void AddCudaIncludeArgs(const llvm::opt::ArgList &DriverArgs,
                            llvm::opt::ArgStringList &CC1Args) const override;
    void AddHIPIncludeArgs(const llvm::opt::ArgList &DriverArgs,
                           llvm::opt::ArgStringList &CC1Args) const override;
    void AddIAMCUIncludeArgs(const llvm::opt::ArgList &DriverArgs,
                             llvm::opt::ArgStringList &CC1Args) const override;
    CXXStdlibType GetDefaultCXXStdlibType() const override;
    bool
    IsAArch64OutlineAtomicsDefault(const llvm::opt::ArgList &Args) const override;
    bool isPIEDefault() const override;
    bool isNoExecStackDefault() const override;
    bool IsMathErrnoDefault() const override;
    SanitizerMask getSupportedSanitizers() const override;
    void addProfileRTLibs(const llvm::opt::ArgList &Args,
                          llvm::opt::ArgStringList &CmdArgs) const override;
    std::string computeSysRoot() const override;

    std::string getDynamicLinker(const llvm::opt::ArgList &Args) const override;

    void addExtraOpts(llvm::opt::ArgStringList &CmdArgs) const override;

    std::vector<std::string> ExtraOpts;

    llvm::DenormalMode getDefaultDenormalModeForType(
        const llvm::opt::ArgList &DriverArgs, const JobAction &JA,
        const llvm::fltSemantics *FPType = nullptr) const override;
    */
 protected:
   Tool *buildAssembler() const override;
   Tool *buildLinker() const override;
   /*Tool *buildStaticLibTool() const override;
   std::string getMultiarchTriple(const Driver &D,
                                 const llvm::Triple &TargetTriple,
                                 StringRef SysRoot) const override;*/
 };
</source>

Note that we're inheriting from the Generic_ELF toolchain, but you can look at some other examples (Windows, Mac OS) for alternatives.

===clang/lib/Driver/ToolChains/Myos.cpp===
Example: https://raw.githubusercontent.com/lexasub/llvm-project-kos/main/clang/lib/Driver/ToolChains/Kolibri.cpp

Here's the code for the toolchain object, insert it somewhere in this file:
<source lang="cpp">
 /// MyOS MyOS tool chain which can call as(1) and ld(1) directly.

MyOS::MyOS(const Driver &D, const llvm::Triple& Triple, const ArgList &Args)
  : Generic_ELF(D, Triple, Args) {
   // Fill this in with your default library paths one day..
   //getFilePaths().push_back(getDriver().Dir + "/../lib");
   //getFilePaths().push_back("/usr/lib");
}

Tool *MyOS::buildAssembler() const {
  return new tools::myos::Assemble(*this);
}

Tool *MyOS::buildLinker() const {
  return new tools::myos::Link(*this);
}
</source>

Note in the constructor that we have the ability to add default include paths, which are sent to our assembler and linker. We'll comment them out now so our system doesn't automatically try to add our host system's libraries when we compile code for our OS.

===clang/lib/Frontend/InitHeaderSearch.cpp===
In the function InitHeaderSearch::AddDefaultCIncludePaths, add this somewhere, so we don't automatically add /usr/local/include as an include path:
<source lang="cpp">
  case llvm::Triple::MyOS:
    // Fill this in with your default include paths...
    // AddPath("/usr/local/include", System, false);
    break;
</source>

If you want your target to automatically add default include paths, you can customize this file. Add your target under AddDefaultCIncludePaths, AddDefaultCPlusPlusIncludePaths, AddDefaultIncludePaths, etc.

===clang/lib/Driver/Driver.cpp===
We need make LLVM use our toolchain object when it targets our OS, so in Driver::getToolChain, add your OS to switch (Target.getOS()):
<source lang="cpp">
...
#include "ToolChains/Hurd.h"
#include "ToolChains/Myos.h"
#include "ToolChains/Lanai.h"
...
    case llvm::Triple::MyOS:
      /*if (Target.getArch() == llvm::Triple::hexagon)
          TC =
            std::make_unique<toolchains::HexagonToolChain>(*this, Target, Args);
        else if ((Target.getVendor() == llvm::Triple::MipsTechnologies) &&
               !Target.hasEnvironment())
          TC = std::make_unique<toolchains::MipsLLVMToolChain>(*this, Target,
                                                             Args);
        else if (Target.isPPC())
          TC = std::make_unique<toolchains::PPCLinuxToolChain>(*this, Target,
                                                             Args);
        else if (Target.getArch() == llvm::Triple::ve)
          TC = std::make_unique<toolchains::VEToolChain>(*this, Target, Args);
        else*/
      TC = std::make_unique<toolchains::MyOS>(*this, Target, Args);
      break;
...
</source>

===clang/lib/Driver/Tools.h===
<b>WHAT FILE IN NEW LLVM?MAY BE NEED REPLACED TO LLD CUSTOM</b>

In here, we define the Assemble and Link classes that our toolchain object references:
<source lang="cpp">
/// myos -- Directly call GNU Binutils assembler and linker
namespace myos {
  class LLVM_LIBRARY_VISIBILITY Assemble : public GnuTool  {
  public:
    Assemble(const ToolChain &TC) : GnuTool("myos::Assemble", "assembler",
                                         TC) {}

    bool hasIntegratedCPP() const override { return false; }

    void ConstructJob(Compilation &C, const JobAction &JA,
                      const InputInfo &Output,
                      const InputInfoList &Inputs,
                      const llvm::opt::ArgList &TCArgs,
                      const char *LinkingOutput) const override;
  };
  class LLVM_LIBRARY_VISIBILITY Link : public GnuTool  {
  public:
    Link(const ToolChain &TC) : GnuTool("myos::Link", "linker", TC) {}

    bool hasIntegratedCPP() const override { return false; }
    bool isLinkJob() const override { return true; }

    void ConstructJob(Compilation &C, const JobAction &JA,
                      const InputInfo &Output,
                      const InputInfoList &Inputs,
                      const llvm::opt::ArgList &TCArgs,
                      const char *LinkingOutput) const override;
  };
} // end namespace myos
</source>

TODO: We're inheriting from GnuTool, use LLD and LLVM-as.

===clang/lib/Driver/Tools.cpp===
<b>WHAT FILE IN NEW LLVM?MAY BE NEED REPLACED TO LLD CUSTOM</b>

Here's the code for our Assemble and Compile - they invoke 'as' and 'ld'.
<source lang="cpp">
 void myos::Assemble::ConstructJob(Compilation &C, const JobAction &JA,
                                   const InputInfo &Output,
                                   const InputInfoList &Inputs,
                                   const ArgList &Args,
                                   const char *LinkingOutput) const {
  ArgStringList CmdArgs;

  Args.AddAllArgValues(CmdArgs, options::OPT_Wa_COMMA, options::OPT_Xassembler);

  CmdArgs.push_back("-o");
  CmdArgs.push_back(Output.getFilename());

  for (const auto &II : Inputs)
    CmdArgs.push_back(II.getFilename());

  const char *Exec = Args.MakeArgString(getToolChain().GetProgramPath("as"));
  C.addCommand(llvm::make_unique<Command>(JA, *this, Exec, CmdArgs));
 }

 void myos::Link::ConstructJob(Compilation &C, const JobAction &JA,
                               const InputInfo &Output,
                               const InputInfoList &Inputs,
                               const ArgList &Args,
                               const char *LinkingOutput) const {
  const Driver &D = getToolChain().getDriver();
  ArgStringList CmdArgs;

  if (Output.isFilename()) {
    CmdArgs.push_back("-o");
    CmdArgs.push_back(Output.getFilename());
  } else {
    assert(Output.isNothing() && "Invalid output.");
  }

  /* if (!Args.hasArg(options::OPT_nostdlib) &&
      !Args.hasArg(options::OPT_nostartfiles)) {
      CmdArgs.push_back(Args.MakeArgString(getToolChain().GetFilePath("crt1.o")));
      CmdArgs.push_back(Args.MakeArgString(getToolChain().GetFilePath("crti.o")));
      CmdArgs.push_back(Args.MakeArgString(getToolChain().GetFilePath("crtbegin.o")));
      CmdArgs.push_back(Args.MakeArgString(getToolChain().GetFilePath("crtn.o")));
  }*/

  Args.AddAllArgs(CmdArgs, options::OPT_L);
  Args.AddAllArgs(CmdArgs, options::OPT_T_Group);
  Args.AddAllArgs(CmdArgs, options::OPT_e);

  AddLinkerInputs(getToolChain(), Inputs, Args, CmdArgs);

  addProfileRT(getToolChain(), Args, CmdArgs);

  if (!Args.hasArg(options::OPT_nostdlib) &&
      !Args.hasArg(options::OPT_nodefaultlibs)) {
    if (D.CCCIsCXX()) {
      getToolChain().AddCXXStdlibLibArgs(Args, CmdArgs);
      CmdArgs.push_back("-lm");
    }
  }

  // We already have no stdlib...
  /*if (!Args.hasArg(options::OPT_nostdlib) &&
      !Args.hasArg(options::OPT_nostartfiles)) {
    if (Args.hasArg(options::OPT_pthread))
      CmdArgs.push_back("-lpthread");
    CmdArgs.push_back("-lc");
    CmdArgs.push_back("-lCompilerRT-Generic");
    CmdArgs.push_back("-L/usr/pkg/compiler-rt/lib");
    CmdArgs.push_back(
         Args.MakeArgString(getToolChain().GetFilePath("crtend.o")));
  }*/

  const char *Exec = Args.MakeArgString(getToolChain().GetLinkerPath());
  C.addCommand(llvm::make_unique<Command>(JA, *this, Exec, CmdArgs));
 }
</source>
TODO: Replace with LLD and LLVM-as.

== Compiling the toolchain ==
    cmake -G <generator> [options] ../llvm
    prefered: cmake -GNijna [options] ../llvm
    Some common build system generators are:
        Ninja --- for generating Ninja build files. Most llvm developers use Ninja.
        Unix Makefiles --- for generating make-compatible parallel makefiles.
        Visual Studio --- for generating Visual Studio projects and solutions.
        Xcode --- for generating Xcode projects.

Some Common options:

-DLLVM_ENABLE_PROJECTS='...' --- semicolon-separated list of the LLVM sub-projects you'd like to additionally build. 
Can include any of: clang, clang-tools-extra, libcxx, libcxxabi, libunwind, lldb, compiler-rt, lld, polly, or debuginfo-tests.

For example, to build LLVM, Clang, libcxx, and libcxxabi, use -DLLVM_ENABLE_PROJECTS="clang;libcxx;libcxxabi".

-DCMAKE_INSTALL_PREFIX=directory --- Specify for directory the full path name of where you want the LLVM tools and libraries to be installed (default /usr/local).
-DCMAKE_BUILD_TYPE=type --- Valid options for type are Debug, Release, RelWithDebInfo, and MinSizeRel. Default is Debug.
-DLLVM_ENABLE_ASSERTIONS=On --- Compile with assertion checks enabled (default is Yes for Debug builds, No for all other build types).

    cmake --build . [-- [options] <target>] or your build system specified above directly.
The default target (i.e. ninja or make) will build all of LLVM.

The check-all target (i.e. ninja check-all) will run the regression tests to ensure everything is in working order.

CMake will generate targets for each tool and library, and most LLVM sub-projects generate their own check-<project> target.

Running a serial build will be slow. To improve speed, try running a parallel build. That's done by default in Ninja; for make, use the option -j NNN, where NNN is the number of parallel jobs, e.g. the number of CPUs you have.

== Installation ==
sudo cmake install

== Compiling your first program for your OS ==
Let's test out the compiler! Create a simple C file:
<source lang="c">
 int do_something(int a, int b) {
   return a * b;
 }
</source>

And compile it with the Clang system you just built: (Use --target=x86_64-myos if you added your OS as a 64-bit platform.)
 clang -target x86-myos -c -o test.o test.c

TODO: The -c option tells us to just compile, because we haven't finished with the linker yet.

This is great, but you will notice you can't just start including <stdio.h> because you'll need to port a C library first. If you don't want to use the C library (you won't be able to use LibC++/the C++ library - and many of the fancier C++11 features that require runtime support) you can stop here.

==Porting a C Library==
At some point you'll likely want a [[C Library]]. Libc++ depends on a functioning C library.

'''TODO''': Any Clang specific stuff here.

== Hosting on our OS ==
This section is for compiling the LLVM toolchain to run under your OS, rather than just build programs for your OS.

'''TODO''': I haven't gotten this far yet!

== External Links ==
* [http://clang.llvm.org/get_started.html Getting Started: Building and Running Clang]
* [http://clang.llvm.org/docs/CrossCompilation.html Cross-compilation using Clang]
* [http://llvm.org/docs/HowToCrossCompileLLVM.html How To Cross-Compile Clang/LLVM using Clang/LLVM]

== See Also ==
* [[OS Specific Toolchain]]
* [[LLVM Cross-Compiler]]
````

<a id="w003695"></a>

## W003695 — Memory Map

Bron-ID: W003695
Namespace: Main (0)
Revisie: 26505 | Datum: 2021-10-15T19:03:02Z
Bron: https://wiki.osdev.org/Memory_Map
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Map&oldid=26505

````text
* [[Memory Map (x86)]]
* [[Memory Map (S390)]]
````

<a id="w003696"></a>

## W003696 — Talk:James Molloy's Tutorial Known Bugs

Bron-ID: W003696
Namespace: Talk (1)
Revisie: 21729 | Datum: 2017-11-21T06:39:19Z
Bron: https://wiki.osdev.org/Talk:James_Molloy%27s_Tutorial_Known_Bugs
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AJames_Molloy%27s_Tutorial_Known_Bugs&oldid=21729

````text
== Problem: Heap Code ==

Maybe we should advise people more, to not use JamesM at all? --[[User:Makerimages|Makerimages]] 13:21, 19 August 2015 (CDT)

"It is probably best that you write your own heap implementation." What's the purpose of this suggestion if it's so vague? Is Molloy's implementation buggy? --[[User:Espanish|Espanish]] 05:58, 6 December 2014‎ (CST) 
:I admit that I didn't read that code too closely. My impression at a glance was that I did not like the code, and didn't want to look for mistakes in it. You also forgot to sign your message by typing “<tt>--</tt><tt>~~</tt><tt>~~</tt>”. --[[User:Sortie|Sortie]] 11:56, 6 December 2014 (CST)

== Nobody noticed that the text for "Problem: __attribute__((packed))" is wrong? ==

The text describes __attribute__((packed)) as the was C/C++ handles structs, packed does _not_ align/pad entries in the struct.
--[[User:Vtlmks|Vtlmks]] 00:39, 21 November 2017 (CST)
````

<a id="w003697"></a>

## W003697 — File:ISBN 0321968972.jpg

Bron-ID: W003697
Namespace: File (6)
Revisie: 17340 | Datum: 2014-12-08T13:48:10Z
Bron: https://wiki.osdev.org/File:ISBN_0321968972.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0321968972.jpg&oldid=17340

````text
The Design and Implementation of the FreeBSD Operating System, 2nd ed.
````

<a id="w003698"></a>

## W003698 — Hard Build System

Bron-ID: W003698
Namespace: Main (0)
Revisie: 29647 | Datum: 2025-09-13T09:28:47Z
Bron: https://wiki.osdev.org/Hard_Build_System
Revisie-URL: https://wiki.osdev.org/index.php?title=Hard_Build_System&oldid=29647

````text
:''Everything should be made as simple as possible, but not simpler.''

Build systems for operating systems implement a complex task and tend to be highly customizable. It's crucial to pick good abstractions to manage this complexity. A source code tree has many aspects of configuration, such as the target processor architecture, what optimizations and warnings are used, where the code is installed, which compiler is used, and so on. No solution fits all.

It's tempting to fall in the trap of providing an easy build system. The user merely runs a script with administrator access, causing system-wide changes to occur, source code to be built, programs installed, and a somehow an operating system image appear on the other end. No effort, just waiting. This approach is too simple and ends up being too complex.

== Example ==

Cross-building a fully-featured Unix-like operating system with an user-space and ports could be done using these steps (requiring decisions):

* Download the source code (toolchain sources, core operating system, ports).
** Decision: What mirrors to download from.
** Decision: Where to install the source code.
* Install required programs as described in the documentation.
** Decision: Where to get the programs from (package management (deb, rpm, ...), build from source, ...).
** Decision: What version of the programs are used.
** Decision: Where to install the programs.
* Build and install custom programs for this operating system.
** Decision: What compiler is used and the compiler options.
** Decision: Where to install the programs.
* Prepare a [[System Root]] containing the standard library headers (needed to make a [[Hosted GCC Cross-Compiler]]).
** Decision: Where the system root is located.
** Decision: What processor architecture is used.
* Build and install a cross toolchain:
** Decision: What compiler and compiler options are used to compile the cross-compiler.
** Decision: Where to install the cross-compiler.
* Build the core operating system and ports and install into the system root.
** Decision: What cross-compiler to use and the compiler options used to compile your operating system and ports.
** Decision: Whether this is a debug build and whether compiler sanitation and security measures are used.
** Decision: What ports is to be built.
* Create a bootable image from the sysroot contents.
** Decision: What boot method is used (cdrom, harddisk, floppy).
** Decision: What to include and what not to (if making a minimal build).
** Decision: What drivers are needed in the initrd.
* Run the operating system in a virtual machine.
** Decision: Which virtual machine is used.
** Decision: What processor features, simulated hardware and how much memory is available.
** Decision: Whether hardware acceleration is enabled.
* Install the operating system on real hardware.
** Decision: What bootloader is used.
** Decision: What harddisk and partition is used.
** Decision: What filesystem is used.

It is obvious there are many more decisions to make and that users will make different decisions. You shouldn't make most of these decisions for the user. It is the user's responsibility to install software and make system-wide changes, not your build system's. Rather, you should document what needs to be done and let the user do it.

== Hard Build System ==

An Hard Build System just builds the operating system. The user prepares the build environment manually by following the documentation, or perhaps using convenience scripts. It trusts the user is able to install prerequisite programs and libraries. It trusts the user to download the necessary source code.

The user is in control, the user decides what to do and does it manually or invokes the convenient scripts if applicable. This is in contrast to the easy build system, where the script is in control and the user might be consulted. The goal isn't to make things hard, but to embrace that particular decisions must be made by the user and make the design no simpler.

=== Help the User ===

Help the user do things:

* Document the process.
* Link to the primary source code mirror.
* Document prerequisite programs and list package names for common Linux distributions.
* Document how to create a cross-compiler (Maybe offer a pre-built toolchain for common systems).
* Have an easy way to create a system root and populate it with standard library headers.
* Default to sane optimization settings if unspecified.

Document the primary way of doing things, and offer secondary short-cuts for those that desire.

=== Cross-Compilation is Hard ===

A cross-compilation inherently have more complexity than a native compilation. For instance, two toolchains exist: The one that makes local programs, and the one that cross-compiles. A [[System Root]] is used. You have to prepare the compilation specially.

The solution is to accept this fact. Cross-compiling your operating system is the secondary way to build your operating system and it can only be so simple. The primary way to build your operating system is on itself. You might not be there yet, but design your build system to run on your own operating system primarily.

Have your build system always compile for the current system by default. This is the natural choice when you primarily develop under your OS. Have the user explicitly request cross-compilation code by setting <tt>CC</tt>, <tt>HOST</tt>, or perhaps passing a --host option to your build system. When your operating system supports multiple processors, it's not clear which platform to target by default when cross-compiling. What if you are on an ARM system compiling your MIPS and x86-64 OS? Solve the issue by targeting the local operating system (even if it is non-sensible to compile your OS with a Linux compiler) and thus forcing the user to specify the intended cross target.

=== Separate the Toolchain ===

When you develop your operating system under itself, you use the system compiler to compile your operating system. The system compiler is merely a program installed in a system directory. It is natural for the same to be the case when cross-compiling: The cross-compiler is just a program installed somewhere.

Store the cross-compiler in a directory the user chooses, such as <tt>$HOME/opt/x86_64-myos</tt> and put its <tt>bin</tt> in the <tt>PATH</tt>. Don't put it inside the operating system's source directory, it's unrelated to that.

The compiler built by your toolchain should be the native compiler on your operating system, cross-compiled by the cross-compiler, when you get this far.

=== Doing is Learning ===

The user learns by having to manually do operations following documentation. This information is crucial in operating system's development, where the developer is required to be an expert to succeed. Being able to make a cross-compiler is great practice for porting a compiler to your OS. Understanding the compilation process is helpful when it fails.

=== Good Users Script ===

You may find that you always make particular decisions (even though some people might take other decisions). In such cases, you can simply write personal scripts that make things easy for you. You can share those scripts, others can be adapt them to their needs or just use them.

=== Configuration is Personal ===

Don't require contributors to edit files that are part of the build system (those that would show as changed by version control if edited) to customize trivial decisions. Environment variables and options passed to the build system are examples of personal configuration that exists outside of the general purpose build system.

For instance, you may be tempted to hard-code the path to the cross-compiler directory in your project's Makefile. Don't do this, instead assume it is already in the <tt>PATH</tt>.

== Easy Build System ==

An Easy Build System is responsible for as much as possible, avoiding any user effort. Degrees of this exist, of course. The ultimate easy build system is when the user blindly pipes an URL into an administrative shell:

<syntaxhighlight lang="bash">
download-to-stdout http://example.com/myos.sh | sudo sh
</syntaxhighlight>

It'll attempt to do as much work as possible to reliably understand the local operating system. It'll figure out what software needs to be installed, detect the distro and install the correct packages, download and compile source code if needed, might make global changes if needed, and generally try to pick sensible defaults. The script will be very complex because the systems out there are very different. The user can merely hope it works and doesn't screw up the local system too much.

=== Interactive Build System ===

A step up from the blind easy build system is the interactive build system. It asks questions underway, such as where the source code should be installed. A good one might ask a lot of questions and present sensible defaults. Such scripts tend to be even more complex than the blind scripts.

An alternate way is for the user to download a script and simply edit variables at the top. This avoids the complexity of interaction, at least.

=== Easy Contributions Fallacy ===

It's easier to attract contributors if it requires less effort for them to build the project. Preferably no effort at all and no decisions on their part. But consider: Are these contributors are worth it? If they are unable to manually build a cross-toolchain, install necessary programs, download the source code, and everything up - will they be able to make any non-trivial contributions to an operating system?

If you really wish to make things easy, instead of supporting the many wildly different systems they normally use, you should become self-hosting. You can then simply give them an image of your operating system containing the development tools and source code already set up perfectly. They can install it locally or run it in a virtual machine.

Perhaps the goal is only that they can try your OS easily? In that case, you simply need to do occasional releases or set up a nightly build service. Building from source is always harder than just running an existing image.

=== No Error Recovery ===

Unfortunately things tend to go wrong when compiling an operating system. A naive script might seriously malfunction if this happens. A reliable script will detect this and simply fail out. This might mean the user has to completely start over, repeating the same work building a cross-compiler, just because you ran out of harddisk space linking the kernel.

A better interactive script might just ask if the user wants to try again or skip to a particular step, at the cost of more script complexity. Compare this with the user just entering commands manually in the shell.

=== Sourcing Scripts ===

Some solutions create a small script that the user sources (a shell equivalent of #include) in the current shell. These scripts usually set a few environment variables and adds stuff to the <tt>PATH</tt>. A hard build system would simply have the user add directories permanently to the <tt>PATH</tt> in <tt>~/.profile</tt> or such.

=== Trust Issues ===

It's harder to trust an easy build system, when you have to comprehend complex shell scripts running as root. It's easier to verify commands you enter yourself, compile as an unprivileged user, read the Makefiles and build scripts before invoking them. It's easier to ensure the easy build system doesn't restart renaming files in <tt>/usr</tt> and other scary things.

== See Also ==

=== Articles ===
* [[Meaty Skeleton]] - A minimal operating system template with a hard build system.
* [[Makefile]] - A makefile tutorial

=== External Links ===
* [http://www.gnu.org/software/make/manual/make.html GNU Make manual] ([http://www.gnu.org/software/make/manual/make.pdf PDF] is [https://www.gnu.org/copyleft/fdl.html freely] available as well)
* [https://sortix.org/man/man7/cross-development.7.html Sortix Cross Development Guide] — a guide on how to build one of the hobby OSes
* [http://aegis.sourceforge.net/auug97.pdf Recursive Make Considered Harmful] — A white paper by Peter Miller on the hazards of using a recursive make build and a suggested alternative  
Note: While the purpose of Peter Miller's paper is sound, the dependency configuration outlined in section 5.4 is considered inferior to the dependency configuration outlined in [[Makefile]].

[[Category:FAQ]]
````

<a id="w003699"></a>

## W003699 — User:Kazinsal/VMWare SVGA

Bron-ID: W003699
Namespace: User (2)
Revisie: 17365 | Datum: 2014-12-11T20:24:45Z
Bron: https://wiki.osdev.org/User:Kazinsal/VMWare_SVGA
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKazinsal%2FVMWare_SVGA&oldid=17365

````text
<center>
{|style="border: 1px solid #cfcfbf; margin-top: 25px; margin-bottom: 25px; background-color: #f0f0ff; text-align: center;"
|
[[image:Under_Construction.png|This page is under construction!]]
|
This page is a work in progress and may thus be incomplete. Its content may be changed in the near future.

<small>This page has notes attached to it: [[User:Kazinsal/VMWare SVGA/Notes]]</small>
|
|}
</center>

Unlike VirtualBox, which emulates enhancements to the Bochs Graphics Adapter, VMWare has its own unique virtual graphics device. Conveniently, the reference implementation of the driver for the VMWare SVGA II device is much more straightforward than that for VBoxVideo and the device has actual documentation (see the Notes page) for both the 2D and 3D graphics functionality.

[[Category:In Progress]]
````

<a id="w003700"></a>

## W003700 — User:Kazinsal/VMWare SVGA/Notes

Bron-ID: W003700
Namespace: User (2)
Revisie: 17364 | Datum: 2014-12-11T20:24:19Z
Bron: https://wiki.osdev.org/User:Kazinsal/VMWare_SVGA/Notes
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKazinsal%2FVMWare_SVGA%2FNotes&oldid=17364

````text
'''Readers: Do not take these notes as fact. These are my musings and such attributable to my downward spiral into insanity while working on the VMWare SVGA research. Proceed at your own risk.'''

* Quick skim of the [http://sourceforge.net/p/vmware-svga/git/ci/master/tree/doc/svga_interface.txt interface spec] says that this is waaaay easier to work with than the VBoxVideo HGSMI system. Modesetting is just as straightforward as in BGA/VBoxVideo.
````

<a id="w003702"></a>

## W003702 — Talk:GCC Cross-Compiler on Gentoo Linux

Bron-ID: W003702
Namespace: Talk (1)
Revisie: 7705 | Datum: 2009-04-19T12:57:40Z
Bron: https://wiki.osdev.org/Talk:GCC_Cross-Compiler_on_Gentoo_Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AGCC_Cross-Compiler_on_Gentoo_Linux&oldid=7705

````text
I just rolled back [http://wiki.osdev.org/index.php?title=GCC_Cross-Compiler_on_Gentoo_Linux&diff=7694&oldid=5670 this edit] by [Imate900]. I can't test it right now, but the crossdev --help says:

<pre>Stage Options:
    -s0, --stage0         Build just binutils
    -s1, --stage1         Also build a C compiler (no libc/C++)
    -s2, --stage2         Also build kernel headers</pre>

If --stage1 only builds binutils for you, that'd be a crossdev bug... are you really sure? -- [[User:Solar|Solar]] 13:46, 17 April 2009 (UTC)
: Silly me... [[User:Imate900|Imate900]] 12:57, 19 April 2009 (UTC)
````

<a id="w003703"></a>

## W003703 — User:Combuster/Alloca

Bron-ID: W003703
Namespace: User (2)
Revisie: 17367 | Datum: 2014-12-13T12:04:56Z
Bron: https://wiki.osdev.org/User:Combuster/Alloca
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster%2FAlloca&oldid=17367

````text
{{Stub}}

==__alloca, ___main==

<tt>alloca()</tt> is a "semi-standard" C function (from BSD?, but supported by most C implementations) that is used to allocate memory from the stack. On Windows this function is also used for stack probing. As such, <tt>alloca()</tt> is referenced in PE binaries built, for example, by Cygwin GCC. You can use the Cygwin GCC argument <tt>-mno-stack-arg-probe</tt> to suppress those references.

Another "specialty" of PE binaries is that, if you define <tt>int main()</tt>, a function <tt>void __main()</tt> is called first thing after entering <tt>main()</tt>. You can either define that function, or omit <tt>main()</tt> from your kernel code, using a different function as entry point.

This explanation of <tt>alloca()</tt> comes from Chris Giese, posted to alt.os.dev:

<blockquote>
>> I think _alloca() is for stack-probing, which is required by Windows.<br>
> What is stack probing?<br>
By default, Windows reserves 1 meg of virtual memory for the stack. No page of stack memory is actually allocated (committed) until the page is accessed. This is demand-allocation.

The page beyond the top of the stack is the guard page. If this page is accessed, memory will be allocated for it, and the guard page moved downward by 4K (one page). Thus, the stack can grow beyond the initial 1MB.

Windows will not, however, let you grow the stack by accessing discontiguous pages of memory. Going beyond the guard page causes an exception. Stack-probing code prevents this.
</blockquote>

Some more information about stack-probing:
* http://groups.google.com/groups?&selm=702bki%24oki%241%40news.Eindhoven.NL.net
* http://groups.google.com/groups?&selm=3381B63D.6E39%40austin.finnigan.com


<tt>alloca()</tt> is not only used for stack-probing, but also as a sort of <tt>malloc()</tt>, to dynamically allocate memory for variables/buffers without the need to manually free the reserved memory (with <tt>free()</tt>, as you normally should).

If you search the man pages for <tt>alloca</tt> on a UNIX OS you'll find something like this:

<blockquote>
The <tt>alloca()</tt> function allocates space in the stack frame of the caller, and returns a pointer to the allocated block. This temporary space is automatically freed when the function from which <tt>alloca()</tt> is called returns.
</blockquote>

On Windows:

<blockquote>
<tt>_alloca</tt> allocates size bytes from the program stack. The allocated space is automatically freed when the calling function exits (not when the allocation merely passes out of scope). Therefore, do not pass the pointer value returned by <tt>_alloca</tt> as an argument to free.
</blockquote>

Note that "a stack overflow exception is generated if the space cannot be allocated", so <tt>alloca()</tt> might cause the stack to grow.

Unfortunately, there are all kinds of caveats with using <tt>alloca()</tt>. Notably, Turbo-C/C++ had the limitation that functions calling <tt>alloca()</tt> needed to have at least one local variable, and Linux man-page warns the following:

<blockquote>
The <tt>alloca</tt> function is machine and compiler dependent. On many systems its implementation is buggy. Its use is discouraged.
</blockquote>

<blockquote>
On  many  systems <tt>alloca</tt> cannot be used inside the list of arguments of a function call, because the stack space reserved by <tt>alloca</tt> would appear  on the stack in the middle of the space for the function arguments.
</blockquote>
````

<a id="w003704"></a>

## W003704 — File:Abbbab.png

Bron-ID: W003704
Namespace: File (6)
Revisie: 17372 | Datum: 2014-12-15T03:52:42Z
Bron: https://wiki.osdev.org/File:Abbbab.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AAbbbab.png&oldid=17372

````text
A simple Finite State Automaton state diagram
````

<a id="w003705"></a>

## W003705 — Lexical Analyzer

Bron-ID: W003705
Namespace: Main (0)
Revisie: 30322 | Datum: 2026-03-17T19:07:57Z
Bron: https://wiki.osdev.org/Lexical_Analyzer
Revisie-URL: https://wiki.osdev.org/index.php?title=Lexical_Analyzer&oldid=30322

````text
{{stub}}

= The Role of the Lexical Analyzer =
In the overwhelming majority of computer languages, a program's source code
takes the form of a stream of text. While this form makes editing a program
a simple matter with conventional tools, an unstructured (from the 
perspective of the compiler) series of characters is not readily parsed
in a meaningful manner. It is the role of the ''lexical analyzer'' to divide up
the character stream into an equivalent stream of ''tokens'' which can be more
easily manipulated and parsed. This is the source of the term ''tokenizer'' as
an alternate name for the lexer.

= Tokens = 
A ''token'' is a classification of a character or group of characters as a unit
that the parser can manipulate. The usual form of a token has two parts, a 
''token type marker'' which describes the token, and a ''literal string'' which
is the actual character sequence from the original character stream. In C,
this can be implemented as an enumeration for the list of token types, and a
struct type for the tokens themselves. To give a simple example of a language
which has one-digit numbers, one letter variables, an addition operator,
a multiplication operator, and left and right parentheses, you might define it
as:
<syntaxhighlight lang="c">
enum TokType
{
    CHAR_LITERAL,
    INT_LITERAL,
    ADDOP,
    MULOP,
    LPAREN,
    RPAREN
};
   
struct Token 
{
     enum TokType token_type;
     char token;
};
</syntaxhighlight>

= Finite State Automata = 
A tokenizer can be written to directly as a case analysis (i.e., a switch
statement or series of if-else if-else statements) that reads each character in
the text, but early in the history of compiler design it was recognized that,
for almost all practical programming languages, the source stream could be
described in terms of a ''regular grammar'', that is, a grammar that can be
expressed in terms of a regular language. A ''regular language'' is a simple
type of language consisting of a series of simple branching and repeating
pathways that can be recognized by an abstraction called a ''Finite State Automaton''.

Finite state automata, or FSAs, are state machines in which there are a fixed
number of possible transitions. In the case of a lexer, each state of the FSA
except the start and end states matches a valid character that can appear in 
the sequence that yields a valid token. For example, if you have a language
in which there can be an 'a', followed by any number of 'b's, followed by 'ab',
then you could describe the language in terms of the following state diagram:

[[file:abbbab.png]]

The trick now becomes applying this diagram form to a program. The simplest way
is to write it as a case analysis, with each possible case testing whether the
character is a valid lexeme in the language, and if it can appear in the
position it shows up in. Each sub-case would be handles by a separate function.

[[Category:Compiler Development]]
````

<a id="w003706"></a>

## W003706 — Talk:AHCI

Bron-ID: W003706
Namespace: Talk (1)
Revisie: 23562 | Datum: 2019-04-08T07:15:38Z
Bron: https://wiki.osdev.org/Talk:AHCI
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAHCI&oldid=23562

````text
== Standard Types ==
This article should be changed to use standard stdint.h and stdbool.h types. --[[User:Sortie|Sortie]] 13:59, 1 January 2015 (CST)

== FIS_DEV_BITS not found anywhere ==
There is no definition in the page and googling it did't find any definition. Can someone fix this ? --[[User:Gabrielravier|GabrielRavier]] 09:14, 8 April 2019 (CEST)
````

<a id="w003708"></a>

## W003708 — User:Sortie/Naively Closed Source

Bron-ID: W003708
Namespace: User (2)
Revisie: 18328 | Datum: 2015-08-30T12:09:20Z
Bron: https://wiki.osdev.org/User:Sortie/Naively_Closed_Source
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie%2FNaively_Closed_Source&oldid=18328

````text
Hobbyist operating system developers often wish to share their work. They do
this for a lot of reasons. They do it to get recognition for their work. They do
it to help others. They do it to taunt others with how far they've come. There's
a lot of reasons and it depends on the developer.

Unfortunately some hobbyist developers are irrationally worried their work will
be exploited unfairly by other people. They make the source code proprietary
(closed-source) just to be safe. There are good reasons to make hobby operating
systems proprietary, but there are a lot of wrong reasons, and beginners tend to
do it for the wrong reasons.

Unless you have good reasons to make your hobbyist operating system proprietary,
you really should [[Licensing|license your OSDev project freely]] and put it in
a public source code repository for everyone to see.

== Fears ==

Hobbyist developers that release their code as proprietary tend to do it because
of these irrational fears:
* Someone might exploit the work commercially without compensating the original author.
* Someone might take credit for the work.
* Someone might fork the code and release it gratis, preventing the original author from commercializing the software.
* Someone might look at the code and not like it (it's not ready for the public).

These fears are often misguided and ignore the advantages of having the source code freely available:
* People can learn from the source code.
* People can use the source code to improve other software.
* People can appreciate the source code.
* People can review your source code for you.
* People can assess your skills and perhaps offer you a job if your code is good.
* People can access the source code without asking you, making it more likely people look at it.

Assuming no one really cares about your OS, you have more to gain from releasing
it freely than keeping it to yourself.

== Nobody really cares about your Hobby OS ==

The main reason the fears are misguided is that nobody actually cares about your
hobby operating system. Many experienced developers have very advanced hobbyist
operating systems and make occasional releases, and they have a very hard time
getting anyone to care. If it hasn't happened to them, and their source code
tend to be freely available, nobody will do it your project.

Nobody is going to pay for your general purpose hobbyist operating system. Some
commercial hobbyist operating systems do exist, and nobody cares. If you can't
make money off it yourself, nobody else is going to commercialize your code.
Besides, if anyone actually wants to commercialize your code, what is the first
obvious thing they do? They hire the original developer, the one that wrote it
all and knows it best.

It's significantly easier to take credit for an existing project, granted, and
it happens now and then with real software problems. Those that do this,
however, tend to be wannabe idiots. Anyone moderately intelligent can see
through this immediately. They definitely can't maintain the project, and those
that pay attention to such feats are probably suckers. Some serious cases happen
where intelligent people reuse source code in their projects and remove all
attribution from the source code. Except small fragments, this is entirely
unacceptable. Free software licenses commonly require copyright notices to stay
intact. If a serious contender takes credit for your work in this manner, you
may be able to take legal action, even if you give the source code freely away.

The fear that someone will continue developing the software freely, preventing
you from commercializing it yourself, is more serious. This is a good reason to
be proprietary. This, however, assumes that you can actually commercialize it,
which you really probably can't.

Perhaps people actually care about your hobbyist operating system?
Congratulations, that's awesome. Your reasons for being proprietary might be good
rather than naive, then.

== All Rights Reserved is Cool ==

Some people pretend to be operating system developers rather than actually being real operating system developers. They see real systems have restrictive copyright notices, horrible end user license agreements, and whatever else - and then emulate it themselves, attempting to be a mini-Microsoft. This is usually immature disillusion thoughts of grandeur. Don't be proprietary because it's cool.

== What if my code is bad? ==

Perhaps you have stage fright? The source code isn't ready, the OSDev gurus
won't like it, it'll be humiliating, it'll damage my online reputation. You're
probably over-thinking this.

Hobbyist operating system development have a very high mortality rate, by even
making this far, you're already in the high percentile of success.
Congratulations. That doesn't mean your code is good. It's probably worthless,
awful code. That's not so bad, actually. Most established operating system
developers don't have that good code either. Those that have good code started
out having bad code. Everyone gets better, and automatically hate their older
code.

If your code is bad, releasing it makes it possible for people to review and
comment on it. Experienced developers can often instantly zoom in on especially
troublesome parts and remark on it. Think of this as an advantage rather than
something undesirable. Use their experience to improve your work.

In fact, it's good that you are worried people won't like aspects of your code.
It means you can emulate an experienced developer, or some rough approximation
thereof, in your head. Listen to that developer and fix your code yourself.

== Show, don't teach ==

This article aims to get over the stage fright and release your work freely and
unashamedly. Do not assume being a moderately successful operating system
development implies your code is that good. Don't fall victim to the
Dunning–Kruger effect (look it up now) and think your code is a good example for
other to follow. It's likely not, ask established ones before making others use
your code as a tutorial. Hobbyist operating system's development has a serious
problem with people presenting bad code and methodology as good ways to OSDev.

== Insulting ==

Some people care more about your source code, if anything, than feature lists,
downloadable images, or even screenshots. To them, an announcement about your OS
that doesn't have any source code is just teasing and unsatisfying. It was the
minimum interest they had in the project, willing to take a cursory glance at
your code, and see if they can spot anything of interest.

== Non-commercial only ==

A compromise could be to release the source code but restrict it heavily. This
is somewhat silly. Nobody really cares about your code, and those that care
about a small part, can't make that part of a larger free software project,
where the ability to resell it is a key licensing requirement.

== Ask Only Source ==

Some wish to help other people but don't want to publish their code publicly.
Instead they post an offer to share the code with anyone who asks nicely. This
is a bit silly, though. If the author is willing to share the code anyway, might
as well get over the stage fright and do it publicly. This also deters people
from taking a quick glance at your code, it's much more work to ask, and asking
also implies that the asker take a good look, which is more than the potential
asker might be willing to do.

== This doesn't apply to you ==

Fortunately, your circumstances happen to be different and this advice doesn't
apply to you. Be sure you do what you do for good reasons.
````

<a id="w003709"></a>

## W003709 — User:No92/ARM Instruction Encoding

Bron-ID: W003709
Namespace: User (2)
Revisie: 25611 | Datum: 2021-02-05T21:14:53Z
Bron: https://wiki.osdev.org/User:No92/ARM_Instruction_Encoding
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ANo92%2FARM_Instruction_Encoding&oldid=25611

````text
{{ Template:In_Progress }}

This page summarizes the encoding of instructions for ARM. These encodings are tested on a Raspberry Pi, which is ARMv6.

== condition codes ==
One of the most awesome features of the ARM instruction set is conditional execution of instructions. These codes are 4-bit values that most instructions support. If the instructions support it, they are placed in bits 31-28.

{| {{wikitable}}
!instruction suffix
!description
!code
|-
|eq
|equals / equals zero
|0000
|-
|ne
|not equal
|0001
|-
|cs / hs
|carry set / unsigned higher or same
|0010
|-
|cc / lo
|carry clear / unsigned lower
|0011
|-
|mi
|negative
|0100
|-
|pl
|positive or zero
|0101
|-
|vs
|overflow
|0110
|-
|vc
|no overflow
|0111
|-
|hi
|unsigned higher
|1000
|-
|ls
|unsigned lower or same
|1001
|-
|ge
|signed greater or same
|1010
|-
|lt
|signed less than
|1011
|-
|gt
|signed greater
|1100
|-
|le
|signed less than or equal
|1101
|-
|al
|always
|1110
|}

No suffix defaults to al (always), which is hexadecimal '0xE'.

== b (branch) and bl (branch with link) ==
This instruction changes the flow of the program by setting pc to a value encoded in the instruction.

==== Encoding ====
{| {{wikitable}}
!bits 31 - 28
!bits 27 - 24
!bits 23 - 0
|-
|condition code
|1010 for b <br> 1011 for bl
|signed 24-bit immediate
|}

The signed 24-bit immediate specifies the number of '''instructions''' (they are 4 bytes each) to go up/down in memory.

==== Pitfalls ====
pc is pointing 8 bytes higher that the address of the current instruction. You have to take this into account when calculating an offset.

== swi (Software Interrupt) ==
swi calls a software interrupt. It is used to provide syscalls. On Linux, the swi number for any syscall is always 0, as the syscall number is in r7.

==== Encoding ====
{| {{wikitable}}
!bits 31-28
!bits 27-24
!bits 23-0
|-
|condition code
|1111
|24-bit immediate
|}

== See also ==
=== External Links ===
* [http://re-eject.gbadev.org/files/armref.pdf a instruction set encoding cheat sheet]
````

<a id="w003710"></a>

## W003710 — I have followed a tutorial and it does not work

Bron-ID: W003710
Namespace: Main (0)
Revisie: 28081 | Datum: 2023-07-09T21:04:15Z
Bron: https://wiki.osdev.org/I_have_followed_a_tutorial_and_it_does_not_work
Revisie-URL: https://wiki.osdev.org/index.php?title=I_have_followed_a_tutorial_and_it_does_not_work&oldid=28081

````text
There are multiple reasons that an operating system based on a tutorial may not work.

== You forgot a step ==
This is most common if you copied and pasted the code without reading the explanation of what that code does. Re-reading the tutorial may help solve the problem.

== Your tools are too old or too new ==
Features are added and removed from devtools all the time. If the tutorial is outdated it may be written for users of an outdated tool like [[TurboC]] or [[DOS]]. If it is too new then you may have be using tools which don't have features that the tutorial uses. If the tutorial is too old then you should find a newer tutorial or write it yourself without using a tutorial. If the tutorial is too new you should update your tools to the latest version and check the "Prerequisites" section in the tutorial if there is one.

== Tutorial bugs ==
There is always the chance that the tutorial might have bugs in it. If the tutorial has not been updated ever, the chance for bugs is higher than if the tutorial is updated regularly. If the tutorial is on the OSDev Wiki, you should check the Discussion tab and also check if the article is disputed. However, tutorial bugs are less common than you think so you should make sure that you did not screw up before you blame the tutorial author.

[[Category:FAQ]]
[[Category:Troubleshooting]]
````

<a id="w003711"></a>

## W003711 — User:Jnc100/UEFI Bare Bones

Bron-ID: W003711
Namespace: User (2)
Revisie: 25175 | Datum: 2020-09-17T15:46:50Z
Bron: https://wiki.osdev.org/User:Jnc100/UEFI_Bare_Bones
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJnc100%2FUEFI_Bare_Bones&oldid=25175
Doorverwijzing: UEFI App Bare Bones

````text
#REDIRECT [[UEFI App Bare Bones]]
````

<a id="w003713"></a>

## W003713 — File:Vector.png

Bron-ID: W003713
Namespace: File (6)
Revisie: 17489 | Datum: 2015-01-24T10:42:41Z
Bron: https://wiki.osdev.org/File:Vector.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVector.png&oldid=17489

````text
Vector desktop demonstrating its window manager and toolkit
````

<a id="w003714"></a>

## W003714 — File:Vector desktop.png

Bron-ID: W003714
Namespace: File (6)
Revisie: 17512 | Datum: 2015-02-02T16:36:31Z
Bron: https://wiki.osdev.org/File:Vector_desktop.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVector_desktop.png&oldid=17512

````text
Vector's GUI showing the CPUID driver
````

<a id="w003716"></a>

## W003716 — User talk:Mrvn

Bron-ID: W003716
Namespace: User talk (3)
Revisie: 17580 | Datum: 2015-02-13T09:42:05Z
Bron: https://wiki.osdev.org/User_talk:Mrvn
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AMrvn&oldid=17580

````text
[[User:Mrvn/Raspberry Pi 2|Raspberry Pi 2]]
````

<a id="w003717"></a>

## W003717 — User:Mrvn/Raspberry Pi 2

Bron-ID: W003717
Namespace: User (2)
Revisie: 17586 | Datum: 2015-02-13T12:54:48Z
Bron: https://wiki.osdev.org/User:Mrvn/Raspberry_Pi_2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMrvn%2FRaspberry_Pi_2&oldid=17586

````text
== Notes about the Raspberry Pi 2 ==

=== Toolchain ===

I'm using binutils-arm-none-eabi and gcc-arm-none-eabi packages from Debian sid, which includes support for a ton of systems. For RPi2 the following options are fitting:

<pre>
--------------------------------------------------------------------
| ARM Core | Command Line Options                       | multilib |
|----------|--------------------------------------------|----------|
|Cortex-A* | [-mthumb] -march=armv7-a -mfloat-abi=hard  | armv7-ar |
|(Hard FP) | -mfpu=vfpv3-d16                            | /thumb   |
|          |                                            | /fpu     |
--------------------------------------------------------------------
</pre>

=== Specs ===

[http://www.arm.com/products/processors/cortex-a/cortex-a7.php official Cortex A7 docs]

PERIPHERAL_BASE = $3F000000 ; Peripheral Base Address (used to be $20000000 on RPi)

LED on GPIO pin 47 (used to be 16 and on/off reversed)


=== Forums ===

[http://www.raspberrypi.org/forums/viewtopic.php?f=72&t=98904 raspberrypi.org forum]

=== Examples ===

[https://github.com/PeterLemon/RaspberryPi/tree/master/VFP/Fractal/Mandelbrot/Double Mandelbrot fractal example]
````

<a id="w003718"></a>

## W003718 — User:Alexis211

Bron-ID: W003718
Namespace: User (2)
Revisie: 17607 | Datum: 2015-02-15T18:49:14Z
Bron: https://wiki.osdev.org/User:Alexis211
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlexis211&oldid=17607

````text
== Current OS projects ==

=== kogata ===

* Written in C
* Monolithic design
* Emphasis on good code quality and reliability
* Goal : small and cool
* [http://adnab.me/cgit/kogata.git/about/ git repo]

Sorry, not very documented yet. I try to have a clean and readable source code though.

== Old OS projects ==

=== Melon ===

* Written in C++
* VESA
* Partial FAT support
* Probably the most advanced of my OS projects
* [https://github.com/Alexis211/Melon GitHub repo]

'''Abandonned''', because I wasn't happy with the system call interface.

=== Grapes ===

* Written in C
* Microkernel
* [https://github.com/Alexis211/Grapes GitHub repo]

'''Abandonned''', because of bad IPC design.

=== T/CE ===

* Based on Grapes
* Hesitation between C and C++
* Not a microkernel anymore
* Playground for various design experiments
* [http://adnab.me/cgit/TCE.git/ git repo]

'''Abandonned''', in favor of new projects with a clearer goal.

== Bona Fide OSDev archives ==

A copy of a few interesting articles from the defunct Bona Fide OS Development website :

* [[User:Alexis211/BonaFideArchive/OS Design]]
* [[User:Alexis211/BonaFideArchive/Device Management]]
````

<a id="w003719"></a>

## W003719 — User:Alexis211/BonaFideArchive/OS Design

Bron-ID: W003719
Namespace: User (2)
Revisie: 17597 | Datum: 2015-02-15T14:28:59Z
Bron: https://wiki.osdev.org/User:Alexis211/BonaFideArchive/OS_Design
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlexis211%2FBonaFideArchive%2FOS_Design&oldid=17597

````text
[https://web.archive.org/web/20120813212303/http://www.osdever.net/tutorials/view/os-design Source]

This comes from an old alt.os.development news thread. The part that has the ">" on each line was written by Ian Nottingham. The rest of it was written by Alistair J. R. Young.

<pre>
> Hey...I've been trying to get started on a kernel for awhile now, but
> all the examples i look at are so far beyond the extreme early stages
> that i can't figure out where to start and what exactally to do when
> it does....anybody have an example of this early stage of development?
> or perhaps a guide or list of what needs to be done? 

The first thing I'd say you need to take into account is that there is
*no* whole subsystem to start with. One of the most fun things about
kernels, I find, is the way that everything has a half-dozen
dependencies on everything else - so you're left with developing
little chunks of a half-dozen subsystems simultaneously.

If you are starting from flat scratch, probably the first thing you
need to develop are a bunch of little macros and functions to put a
prettier face on the more messy bits of the x86 architecture. If
you're really ambitious, you can try and make it a portable or at
least semi-portable face. :)

(I really believe in cheating when it comes to these boring minutiae -
if you're not all that interested in writing lots of low-level
PC-hardware-bashing assembler code, I suggest you get the Utah Oskit
{just released in v. 0.96, woo!} where that bit's all been done for
you.)

You'll also need a bunch of library functions for the kernel (no
strcmp(), no strcpy(), no malloc(), etc. of course, so...?) - plus
debugging functions along the lines of assert() and panic().

At the same time, you *really* need to make your major design
decisions now, otherwise you'll end up in the same situation as I was
- and having to rip the thing out and do it over again is a *major*
pain.

The first big one's the memory model. Segmentation and/or paging? It
seems, these days, that most people go for paging w/o segmentation -
both because the flat memory model's neater, and because paging's
generally portable and segmentation isn't.

And where do you put the kernel in it? Do you allow a separate address
space for each process, or all mixed together? Virtual memory?

Then there's multitasking/threading, if you decide to have
them. Perhaps most crucially at the start, is it possible your code
will ever be running on a SMP machine? If so, put in all the spinlocks
and other synchronisation stuff you'll need right from the start -
doing it in retrospect is an *incredible* nightmare.

What's your process model? Just the basic two-level
processes-owning-threads, or something more sophisticated? (I'm using
a three-level InteractiveSession/Job/Facility owns Process owns Thread
model, myself.) When you do the task-switching, are you going to write
your own code to do it, or use the Intel-provided TSSes to do it. If
the latter, are you going to go the whole way and have one TSS per
thread, or have two and swap the appropriate information into and out
of the inactive one before and after every task switch?

System calls. How do you do them? Several possibilities - Linux, et
al. seem to use software interrupts, so that's quite popular. Other
options - call gates to let a user-mode program directly call the
appropriate kernel functions. Or just have one call to send messages
to IPC ports, or some such, and run all communication even with the
kernel over IPC.

And speaking of IPC - Message ports? Named pipes? Queues? Local
sockets? All of the above? Best to think about it now, because it can
influence the rest of the design quite a bit - you may want to have
one fundamental one and implement everything else in terms of it (in
Laura, for example, everything's pretty much a derivative of the basic
message port - which influences the design of the TCP/IP stack
(networking - something else to think about).

Ich. Device drivers - how do you talk to them? And the
filesystem. This is all one muxed-up topic, really. Simplistically,
there's always the Unix model where you treat everything as a file and
access it all through the filesystem. Or there are more exotic options
waiting out there - again to use my own kernel as an example, the
Laura model has a 'namespace manager' which lists every object on the
system - devices, filesystems and their files, etc., etc. To access
anything, device, file, whatever, you ask the namespace manager and it
gives you back an (harking back to above) an IPC port for that object
- and each registered thing can support a totally customised set of
actions, not just the standard open(), close(), read(), write(),
ioctl()...

Security! If you're going to have any, think about it now. It's almost
worse than SMP to retrofit, at least if you want to avoid having
security holes everywhere. And there are so many different ways to do
it - mine uses object ACLs validated against a possible three
different tokens depending on what the relevant thread, task and
session are doing.

Well, that's all design stuff, but that's the most important in my
book. Getting it done first, I mean. Otherwise, you end up spending a
lot of time ripping out and rewriting.

Once you actually start putting code to keyboard... well, (unless
you're using something that gives you one), about the first thing you
need is a crude console driver. It almost certainly won't be the one
you use to drive the console in the final revision, but trying to
debug anything without printf() is an absolute bastard...

Then - let me see. It depends a lot on how your OS is laid out (design
again :) ) and what the dependencies are. The first thing you need,
obviously, is code that boots the kernel (if you use the MultiBoot
boot loader, or something similar, this is rather less painful than
otherwise. It might be an idea to use something like it to start with
even if you plan to write your own boot-loader later) and does the
most basic setup (get the segments set right, allocate a kernel heap
and stack to work with, and so on and so forth.)

Then, my first step was to write something that displayed a startup
banner and halt (so I could actually see something happening - always
a plus!). Then, building on that - let's see, my first step was to
rewrite that to display the banner and then enter the system idle loop
(the loop that will eventually become the idle thread, that is). That
*is* actually more complicated than just 'for (;;) ;' - as I think
it's a good idea to halt the processor during it to avoid burning too
much power, but...

I think the next thing that's probably a good idea to get working is
trap handling - even though, at this stage, about all that you can do
is display a message, dump state, and panic the kernel (this should,
IMO, be your default handler for everything). The messages will come
in handy when debugging later (much more useful than just a crash),
and it's a sight quicker to implement the real handlers when you've
already got a framework.

The same thing goes for interrupt handling, which I think is probably
the next thing that needs to go on the list. I wouldn't panic on an
unexpected interrupt, though.

After that - well, after that it gets very dependent on the kernel
you're doing. My next step was to get a handler running for the timer
interrupt - just about everything uses the system timer for
*something*, so it's pretty essential. And, also, I need that
working for the scheduler to work, so...
</pre>
````

<a id="w003720"></a>

## W003720 — User:Alexis211/BonaFideArchive/Device Management

Bron-ID: W003720
Namespace: User (2)
Revisie: 17605 | Datum: 2015-02-15T15:02:07Z
Bron: https://wiki.osdev.org/User:Alexis211/BonaFideArchive/Device_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AAlexis211%2FBonaFideArchive%2FDevice_Management&oldid=17605

````text
[https://web.archive.org/web/20120815033601/http://www.osdever.net/tutorials/view/device-management Source]

Tim Robinson · timothy.robinson@ic.ac.uk · http://www.themoebius.org.uk/

== Introduction ==

I'm writing this device management tutorial assuming that you've got a basic working protected-mode kernel. You don't need to have read my memory management tutorials (numbers one and two) but, if not, I recommend that you do. I'll assume that your kernel has a basic operating environment: a working memory manager and basic run-time library facilities.

I'm going to concentrate on designing your device manager instead of telling you things like what order to program the ATA controller registers. You can get hardware information from manufacturers' specifications or from reading other drivers' source code; however, there are very few tutorials which teach you the overall picture, and there are a lot of badly-designed device interfaces around.

== Architectures ==

This is where the various different kernel architectures become apparent. There are two main categories:

* '''Microkernel''' (examples: Minix, QNX)<br />Device drivers are isolated from the kernel, and are usually in their own user-mode address spaces. They communicate with the main kernel and with each other by means of messaging. Passing a message from, say, the kernel to a driver usually involves a process switch (switching address spaces) and a context switch (switching threads) which incurs a relatively high speed penalty. Microkernels' main advantage is stability: if one buggy device driver crashes, all that is affected is access to that device (until the driver is restarted).

* '''Monolithic Kernel''' (examples: Linux, Windows NT)<br />Device drivers run as part of the kernel, either compiled in or as run-time loadable modules. Monolithic kernels have the advantage of speed and efficiency: calls to driver functions are simple local calls instead of whole address space switches. However, because it is running in kernel mode, a buggy device driver has the capability to crash the entire system.

The monolithic architecture is probably the most common in today's operating systems because it is usually more efficient. Monolithic kernels can have access to all the kernel's code and data, making it possible to share internal functions and data with device drivers: Windows NT use this to allow drivers access to the kernel run-time library and Linux allows drivers to access pretty much any non-static kernel symbol. By contrast, microkernel drivers are fully self-contained applications, which makes it a lot harder for them to share common functions. Note that the microkernel separation between the kernel and the rest of the OS components isn't restricted to device drivers: for example, Minix has its file system driver and memory manager as separate tasks too. I'll be concentrating on monolithic architectures in this tutorial.

== Specifications ==

Your device manager will be the interface between the device drivers and the both the rest of the kernel and user applications. It needs to do two things:

* Isolate devices drivers from the kernel so that driver writers can worry about interfacing to the hardware and not about interfacing to the kernel
* Isolate user applications from the hardware so that applications can work on the majority of devices the user might connect to their system

In most operating systems, the device manager is the only part of the kernel that programmers really see. Writing a good interface will make the difference between an efficient and reliable OS which works with a variety of devices and an OS which you spend all your own time writing and debugger drivers for.

Here's the capabilities our device manager will have:

* Asynchronous I/O: that is, applications will be able to start an I/O operation and continue to run until it terminates. This is instead of blocking I/O, whereby applications are stalled while I/O operations execute. Blocking I/O can be easily implemented as a special case of asynchronous I/O.
* Plug and Play: drivers will be able to be loaded and unloaded as devices are added to and removed from the system. Devices will be detected automatically on system startup, if possible.

== Drivers ==

Because we want our kernel to be plug-and-play capable, it isn't enough for drivers to be added to the kernel at compile time, as Minix and old Linux do. We must be able to load and unload them at run time. This isn't difficult: it just means we have to extend the executable file interface to kernel mode.

You can implement kernel driver modules any way you like. Linux uses object files (.o) which are linked to the kernel at load time: references and relocations are patched as if the module was being linked by a linker. Windows NT (and WDM, the Windows Driver Model used by Windows since Windows 98/2000) uses full PE dynamic link libraries with the .sys extension. Drivers are loaded into kernel space just like DLLs are loaded into user space. The kernel needs some kind of entry point into the driver; once it has been loaded, the kernel will be able to see all of the driver's code and data (and vice versa). It needs some way of invoking the driver's code and starting the ball rolling.

This is where device enumeration comes in. How does the kernel know which drivers to load? The same driver could be used for several devices: how does the driver know which devices to implement? Linux, Minix and pre-WDM Windows NT avoid this problem by requiring drivers to know how to detect their own hardware. This makes sense, particularly for the pre-PnP systems which were around when these operating systems were designed. The kernel loads all the drivers which it expects to be needed (often via a user-configurable script or Registry) and the drivers take care of notifying the kernel of which devices actually are present.

Today's PCs have more sophisticated hardware:

* A PCI bus makes hardware detection easily: each type of device responds to a unique combination of a 16-bit vendor ID and a device ID.
* An ISA PnP chipset, together with a PnP BIOS, allows a list of installed ISA PnP devices to be compiled.
* An ACPI chipset provides consistent access to all the motherboard devices.
* A USB hub allows the devices connected to it to be queried for their types and capabilities.

So it is possible to write bus drivers for each type, each of which is able to detect the devices connected to that bus. It may even be possible to write an über-bus driver which can detect the buses installed in a machine, or is might be sufficient for the user to specify the buses installed (or even load each bus driver blindly and get it to run its detection routines regardless). Old-fashioned jumpered devices can even have their own virtual bus driver which stores configuration information on disk instead of getting it from the hardware.

If you write bus drivers for each type you need to put an interface in device drivers which defines a callback to be called for each device supported: an "add-device" routine. If not, you need a "main" routine in each driver which is called by the kernel when the driver is loaded and which detects the devices supported.

== Interfaces ==

Once we've detected the devices installed in the system we need to keep a record of them somewhere. The standard Unix model, employed by Minix and Linux, is to keep a <code>/dev</code> directory somewhere in the file system. This directory is filled with special directory entries - directory entries which don't point to any data - each of which refers to a specific device via major and minor device numbers. The major device number specifies the device type or driver to use, and the minor number specifies a particular device implemented by that driver. For example, all IDE devices in the system might share one major device number, and each of the minor numbers would refer to one of the machine's hard disks, CD-ROMs or tape drives. Because entries in the <code>/dev</code> directory are persistent between reboots (since they are stored on disk), major device numbers never change and minor device numbers are always allocated the same way. The advantage of keeping device links in a directory on disk is that devices appear as files, making it easy to, say, record waveform data from the microphone (<code>/dev/dsp</code>), pipe it through a filter, and play it back through the speakers. All of this could be accomplished on one command line. BeOS also uses a format similar to this - it has a <code>/dev</code> file system - although devices are stored dynamically and are organized in a tree of directories. Although Linux's use of a static <code>/dev</code> directory doesn't encourage the ability to dynamically add and remove devices, the BeOS <code>devfs</code> is fully dynamic and reflects the current configuration of the system without having to tweak the special directory entries.

On the face of it, Windows NT doesn't expose its devices to the user in the same way as Unix does; a user browsing through Explorer would only see the old MS-DOS-style C:, D: etc. volumes. Internally, however, Windows NT makes its devices available in a way similar to Unix. There exists a kernel namespace separate to the file system through which the system's devices can be accessed like ordinary files (actually, the normal file system can be accessed via the kernel namespace - the everyday C: drive names are simply links to the physical disk device names) . The Unix and Windows NT systems have one thing in common: once you know the name, a device can be opened and manipulated from user applications: not only with the stream-orientated <code>read()</code> and <code>write()</code> (or <code>ReadFile()</code> and <code>WriteFile()</code> under Win32) but as a device in its own right, with <code>ioctl()</code> (or <code>DeviceIoControl()</code>).

This leads us to the interface between user applications and device drivers. Typically a user app will issue commands to a specific device via the APIs I mentioned just now. It traps into the kernel via a syscall and the kernel picks up the device requested via the file handle given (the kernel will have recorded the device requested somehow when the file was opened). It is then ready to pass the request onto the driver responsible for that device.

The interface between the kernel and the driver is usually that of a table of function pointers. Linux and Windows NT both use a structure representing a device which contains a series of pointers to functions for handling requests to open, close, read, write, etc. On Linux, each pointer generally corresponds to an individual device syscall, right down to the function prototype; on Windows NT, the pointers each point to generic device control routines. Different routines can be used for each function, but usually similar tasks (e.g. reading and writing) go to the same routine: the driver can tell the requests apart by the information in the I/O Request Packet (IRP) passed to the function.

== Asynchronous I/O ==

Here's the Linux and NT models start to differ. As I mentioned, on receipt of, say, a read syscall, Linux passes control to the driver's read routine. This routine can handle the request immediately and return (as a ramdisk would), or it can start processing the request and put the current application to sleep (as an IDE driver would).

On receipt of a read request, a ramdisk driver would typically:

# Copy memory from the ramdisk to user space
# Return to user mode

An IDE driver would typically:

# Receive the read request and put it on a queue
# If the queue was empty, issue the Read Sectors command to the controller
# Put the task to sleep
# When the IDE interrupt is received, transfer the data from the controller and awaken the task
# When the task awakes control is returned to user mode

The point to note here is that, as far as user code is concerned, the <code>read()</code> call doesn't return until the hard disk has finished reading - it doesn't know that other tasks are being scheduled in the meantime. It ''definitely'' doesn't regain control until it has received the data and an error/success code.

Windows NT is different because it uses an asynchronous I/O model internally. Although the synchronous <code>ReadFile()</code> and <code>WriteFile()</code> routines are most commonly used, the kernel will return control to the application just after the request is started. The overlapped <code>ReadFileEx()</code> and <code>WriteFileEx()</code> routines reflect more accurately the way Windows NT works internally (although on Windows 95 these two functions don't work in the majority of cases).

Although Windows NT's asynchronous I/O model is more complex than Linux's it suits NT's multi-threaded operating environment better. An IDE read request on Windows NT might involve the following:

* '''App:''' Call overlapped <code>ReadFileEx()</code>.
* '''Driver:''' Receive the read IRP from the kernel and put it on a queue.<br />Lock the user buffer in memory and obtain its associated physical addresses.<br />If the queue was empty, issue the Read Sectors command to the controller.<br />Return to user mode.
* '''App:''' Wait for I/O completion: either go to sleep or do other things.
* '''Driver:''' When the IDE interrupt is received, transfer the data from the controller to the physical addresses recorded earlier.<br />Notify the I/O manager that the IRP has completed.
* '''App:Completes wait.

An apparently synchronous I/O request still goes through the same procedure, because for blocking I/O the requesting thread is put to sleep until the operation is finished.

Notice that the driver must lock the buffer the user app provided before it queues the request. This highlights an important point with any multi-tasking architecture: the interrupt which reads the data into the buffer can execute under any context. This means that the kernel could start an operation on behalf of an application and switch away from it before the operation is completed (imagine if an IDE hard drive had been put to sleep before a read occurred: to spin it up could take several seconds, during which time the scheduler might be called thousands of times). So once the controller triggers its interrupt the processor could be executing anything, which means that the current address space is not necessarily the same one from where the request originated. The original buffer is inaccessible, at least in the virtual address space. What we must do is save the physical memory addresses associated with the buffer when the request starts (and lock it in memory, to prevent it from being paged out). Then, when the interrupt handler executes, it can map in the physical pages temporarily and place the data there. Note that as the buffer is locked the kernel must make sure that it is all committed: it's no use getting the physical addresses for a block of memory if it doesn't exist in physical memory yet. Note also that a physical address must be obtained for every virtual page in the buffer, because the pages aren't necessarily contiguous in RAM.

The physical addresses obtained can also be useful for devices that use DMA, such as floppy and hard drive controllers and sound cards, because such devices could potentially transfer data straight into the user buffer. However most DMA drivers prefer to use their own static buffers, and copy the data between the user's and their own buffers as necessary. This is because various devices place restrictions on DMA transfer buffers. The buffer must:

* Be contiguous in memory, unless a scatter-gather DMA device is used (which can be programmed with a list of addresses)
* Be located completely in the first 16MB of memory
* Not span a 64KB boundary

The first one applies to PCI (all of these apply to ISA); if you want to use DMA in your drivers you need to write a physical memory allocator which satisfies these conditions. I wrote about this in my first memory tutorial.

== Overview ==

That's by no means everything you'll need by way of device management, but it should be enough to give you a good start. You should now be able to design a good device driver interface for your kernel: the best designs are not the ones which are immediately brilliant, but which leave the most scope for future enhancement. As your kernel grows you'll be glad of a device interface which doesn't need to be redesigned every 6 months.
````

<a id="w003722"></a>

## W003722 — File:Wataha 0.328.jpg

Bron-ID: W003722
Namespace: File (6)
Revisie: 17620 | Datum: 2015-02-17T20:22:58Z
Bron: https://wiki.osdev.org/File:Wataha_0.328.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWataha_0.328.jpg&oldid=17620

````text

````

<a id="w003723"></a>

## W003723 — File:Wataha.net 0.312.gif

Bron-ID: W003723
Namespace: File (6)
Revisie: 17623 | Datum: 2015-02-17T20:28:12Z
Bron: https://wiki.osdev.org/File:Wataha.net_0.312.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AWataha.net_0.312.gif&oldid=17623

````text

````

<a id="w003724"></a>

## W003724 — User:Bandrami

Bron-ID: W003724
Namespace: User (2)
Revisie: 17630 | Datum: 2015-02-19T03:16:41Z
Bron: https://wiki.osdev.org/User:Bandrami
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ABandrami&oldid=17630

````text
I enjoy making OS-less Forth systems for ARM. If I ever get to the point of naming and releasing one I'll announce it.
````

<a id="w003725"></a>

## W003725 — Implications of writing a freestanding C project

Bron-ID: W003725
Namespace: Main (0)
Revisie: 29429 | Datum: 2025-03-06T11:59:57Z
Bron: https://wiki.osdev.org/Implications_of_writing_a_freestanding_C_project
Revisie-URL: https://wiki.osdev.org/index.php?title=Implications_of_writing_a_freestanding_C_project&oldid=29429

````text
When building an operating system or other freestanding project (that is, a project which for whatever reason cannot link to any of the [[C Library|standard C libraries]]), you need to remember not to use any of those library functions in the project. Any code that requires functions for which the standard C library is normally used - including all the common stream I/O functions - must be provided as part of the project, or as system-specific library or system calls.

In the case of an operating system, porting the standard libraries is usually a priority, but this leads to a slightly different situation where the compiler on the development host must know to link the correct version of the library for the target system. The usual solution to this is to set up a separate [[GCC Cross-Compiler|cross compiler]] for compiling the OS code when the system is not self-hosting. Furthermore, this means that the new system must provide primitive system calls that those libraries can in turn use; the OS itself cannot use the standard library at all, and it is recommended that the OS-specific functions have different names from the standard ones to avoid confusion. This is true for any compiler targeting a different platform or OS, not just GCC; GCC is simply the most common case in OS dev.

== Freestanding Headers ==
The C standard defines freestanding headers. They usually come along with your compiler and can be used in freestanding mode. These headers define data structures, data types, and macros without adding any function prototypes or references to standard functions.

=== Headers available as of C89 ===
[http://en.wikipedia.org/wiki/Float.h <float.h>]<br>
[http://en.wikipedia.org/wiki/Limits.h <limits.h>]<br>
[http://en.wikipedia.org/wiki/Iso646.h <iso646.h>]<br>
[http://en.wikipedia.org/wiki/Stdarg.h <stdarg.h>]<br>
[http://en.wikipedia.org/wiki/Stddef.h <stddef.h>]<br>

=== Headers available as of C99 ===
[http://en.wikipedia.org/wiki/Stdbool.h <stdbool.h>]<br>
[http://en.wikipedia.org/wiki/Stdint.h <stdint.h>]<br>

=== Headers available as of C11 ===
<stdalign.h><br>
<stdnoreturn.h><br>

=== Headers available as of C23 ===
<stdckdint.h><br>
<stdbit.h><br>

[[Category:C]]
[[Category:Standard Libraries]]
````

<a id="w003726"></a>

## W003726 — User:Dallinac

Bron-ID: W003726
Namespace: User (2)
Revisie: 17633 | Datum: 2015-02-19T06:30:46Z
Bron: https://wiki.osdev.org/User:Dallinac
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ADallinac&oldid=17633

````text
Hi! I'm dallinac! I'm working on an operating system with the code name "Stormy" and am pretty brand new to OS development (not a total noob, but new)

==Contributions==
[[Implications of writing a freestanding C project]] - wrote
````

<a id="w003729"></a>

## W003729 — CPU Registers x86

Bron-ID: W003729
Namespace: Main (0)
Revisie: 30487 | Datum: 2026-04-16T09:34:41Z
Bron: https://wiki.osdev.org/CPU_Registers_x86
Revisie-URL: https://wiki.osdev.org/index.php?title=CPU_Registers_x86&oldid=30487

````text
== General Purpose Registers ==

{| {{wikitable}}
|-
! 64-bit
! 32-bit
! 16-bit
! 8 high bits
! 8 low bits
! Description
|-
| RAX
| EAX
| AX
| AH
| AL
| Accumulator
|-
| RBX
| EBX
| BX
| BH
| BL
| Base
|-
| RCX
| ECX
| CX
| CH
| CL
| Counter
|-
| RDX
| EDX
| DX
| DH
| DL
| Data
|-
| RSI
| ESI
| SI
| N/A
| SIL
| Source
|-
| RDI
| EDI
| DI
| N/A
| DIL
| Destination
|-
| RSP
| ESP
| SP
| N/A
| SPL
| Stack Pointer
|-
| RBP
| EBP
| BP
| N/A
| BPL
| Stack Base Pointer
|-
|}

== Pointer Registers ==

{| {{wikitable}}
|-
! 64-bit
! 32-bit
! 16-bit
! Description
|-
| RIP
| EIP
| IP
| Instruction Pointer
|-
|}

== Segment Registers ==

{| {{wikitable}}
|-
! 16-bit
! Description
|-
| CS
| Code Segment
|-
| DS
| Data Segment
|-
| ES
| Extra Segment
|-
| SS
| Stack Segment
|-
| FS
| General Purpose F Segment
|-
| GS
| General Purpose G Segment
|-
|}

== EFLAGS Register ==

{| {{wikitable}}
|-
! Bit
! Label
! Description
|-
| 0
| CF
| Carry flag
|-
| 2
| PF
| Parity flag
|-
| 4
| AF
| Auxiliary flag
|-
| 6
| ZF
| Zero flag
|-
| 7
| SF
| Sign flag
|-
| 8
| TF
| Trap flag
|-
| 9
| IF
| Interrupt enable flag
|-
| 10
| DF
| Direction flag
|-
| 11
| OF
| Overflow flag
|-
| 12-13
| IOPL
| I/O privilege level
|-
| 14
| NT
| Nested task flag
|-
| 16
| RF
| Resume flag
|-
| 17
| VM
| Virtual 8086 mode flag
|-
| 18
| AC
| Alignment check
|-
| 19
| VIF
| Virtual interrupt flag
|-
| 20
| VIP
| Virtual interrupt pending
|-
| 21
| ID
| Able to use CPUID instruction
|-
|}

Unlisted bits are reserved.

==Control Registers==

====CR0====

{| {{wikitable}}
|-
! Bit
! Label
! Description
|-
| 0
| PE
| Protected Mode Enable
|-
| 1
| MP
| Monitor co-processor
|-
| 2
| EM
| x87 FPU Emulation
|-
| 3
| TS
| Task switched
|-
| 4
| ET
| Extension type
|-
| 5
| NE
| Numeric error
|-
| 16
| WP
| Write protect
|-
| 18
| AM
| Alignment mask
|-
| 29
| NW
| Not-write through
|-
| 30
| CD
| Cache disable
|-
| 31
| PG
| Paging
|-
|}
NOTE: This register is the only control register that can be written and read via 2 ways unlike the other that can be accessed only via the MOV instruction
<syntaxhighlight lang="asm">
; First way:
; Write:
mov cr0, reg

; Read:
mov reg, cr0
; ----------------------
; Second way:
; Write:
lmsw reg

; Read:
smsw reg
</syntaxhighlight>

====CR1====

Reserved, the CPU will throw a #UD exception when trying to access it.

====CR2====

{| {{wikitable}}
|-
! Bit
! Label
! Description
|-
| 0-31 (63)
| PFLA
| Page Fault Linear Address
|-
|}

====CR3====

{| {{wikitable}}
|-
! Bit
! Label
! Description
! [[PAE]]
! [[Long Mode]]
|-
| 3
| PWT
| Page-level Write-Through
| (Not used)
| (Not used if bit 17 of CR4 is 1)
|-
| 4
| PCD
| Page-level Cache Disable
| (Not used)
| (Not used if bit 17 of CR4 is 1)
|-
| 12-31 (63)
| PDBR
| Page Directory Base Register
| Base of PDPT
| Base of PML4T/PML5T
|-
|}
Bits 0-11 of the physical base address are assumed to be 0. Bits 3 and 4 of CR3 are only used when accessing a PDE in 32-bit paging without PAE.

====CR4====

{| {{wikitable}}
|-
! Bit
! Label
! Description
|-
| 0
| VME
| Virtual 8086 Mode Extensions
|-
| 1
| PVI
| Protected-mode Virtual Interrupts
|-
| 2
| TSD
| Time Stamp Disable
|-
| 3
| DE
| Debugging Extensions
|-
| 4
| PSE
| Page Size Extension
|-
| 5
| PAE
| Physical Address Extension
|-
| 6
| MCE
| Machine Check Exception
|-
| 7
| PGE
| Page Global Enabled
|-
| 8
| PCE
| Performance-Monitoring Counter enable
|-
| 9
| OSFXSR
| Operating system support for FXSAVE and FXRSTOR instructions
|-
| 10
| OSXMMEXCPT
| Operating System Support for Unmasked SIMD Floating-Point Exceptions
|-
| 11
| UMIP
| User-Mode Instruction Prevention (if set, #GP on SGDT, SIDT, SLDT, SMSW, and STR instructions when CPL > 0)
|-
| 12
| LA57
| 57-bit linear addresses (if set, the processor uses 5-level paging otherwise it uses uses 4-level paging)
|-
| 13
| VMXE
| Virtual Machine Extensions Enable
|-
| 14
| SMXE
| Safer Mode Extensions Enable
|-
| 16
| FSGSBASE
| Enables the instructions RDFSBASE, RDGSBASE, WRFSBASE, and WRGSBASE 
|-
| 17
| PCIDE
| PCID Enable
|-
| 18
| OSXSAVE
| XSAVE and Processor Extended States Enable
|-
| 20
| SMEP
| [[Supervisor Memory Protection|Supervisor Mode Execution Protection]] Enable
|-
| 21
| SMAP
| [[Supervisor Memory Protection|Supervisor Mode Access Prevention]] Enable
|-
| 22
| PKE
| Protection Key Enable
|-
| 23
| CET
| Control-flow Enforcement Technology
|-
| 24
| PKS
| Enable Protection Keys for Supervisor-Mode Pages
|-
|32
|FRED
|[[Flexible Return and Event Delivery]] Enable
|}

====CR5 - CR7====

Reserved, same case as CR1.

====CR8====
{| {{wikitable}}
|-
! Bit
! Label
! Description
|-
| 0-3
| TPL
| Task Priority Level
|-
|}

==Extended Control Registers==

====XCR0====
{| {{wikitable}}
|-
! Bit
! Label
! Description
|-
| 0
| X87
| x87 FPU/MMX support (must be 1)
|-
| 1
| SSE
| XSAVE support for MXCSR and XMM registers
|-
| 2
| AVX
| AVX enabled and XSAVE support for upper halves of YMM registers
|-
| 3
| BNDREG
| MPX enabled and XSAVE support for BND0-BND3 registers
|-
| 4
| BNDCSR
| MPX enabled and XSAVE support for BNDCFGU and BNDSTATUS registers
|-
| 5
| opmask
| AVX-512 enabled and XSAVE support for opmask registers k0-k7
|-
| 6
| ZMM_Hi256
| AVX-512 enabled and XSAVE support for upper halves of lower ZMM registers
|-
| 7
| Hi16_ZMM
| AVX-512 enabled and XSAVE support for upper ZMM registers
|-
| 9
| PKRU
| XSAVE support for PKRU register
|-
|}

XCR0 can only be accessed if bit 18 of CR4 is set to 1. XGETBV and XSETBV instructions are used to access XCR0.

==Debug Registers==

====DR0 - DR3====

Contain linear addresses of up to 4 breakpoints. If paging is enabled, they are translated to physical addresses.

====DR6====

It permits the debugger to determine which debug conditions have occurred.<br>
Bits 0 through 3 indicates, when set, that it's associated breakpoint condition was met when a debug exception was generated.<br>
Bit 13 indicates that the next instruction in the instruction stream accesses one of the debug registers.<br>
Bit 14 indicates (when set) that the debug exception was triggered by the single-step execution mode (enabled with TF bit in EFLAGS).<br>
Bit 15 indicates (when set) that the debug instruction resulted from a task switch where T flag in the TSS of target task was set.<br>
Bit 16 indicates (when clear) that the debug exception or breakpoint exception occured inside an RTM region.<br>

====DR7====

{| {{wikitable}}
|-
! Bit
! Description
|-
| 0
| Local DR0 breakpoint
|-
| 1
| Global DR0 breakpoint
|-
| 2
| Local DR1 breakpoint
|-
| 3
| Global DR1 breakpoint
|-
| 4
| Local DR2 breakpoint
|-
| 5
| Global DR2 breakpoint
|-
| 6
| Local DR3 breakpoint
|-
| 7
| Global DR3 breakpoint
|-
| 16-17
| Conditions for DR0
|-
| 18-19
| Size of DR0 breakpoint
|-
| 20-21
| Conditions for DR1
|-
| 22-23
| Size of DR1 breakpoint
|-
| 24-25
| Conditions for DR2
|-
| 26-27
| Size of DR2 breakpoint
|-
| 28-29
| Conditions for DR3
|-
| 30-31
| Size of DR3 breakpoint
|-
|}
A local breakpoint bit deactivates on hardware task switches, while a global does not.<br>
Condition 00b means execution break, 01b means a write watchpoint, and 11b means an R/W watchpoint. 10b is reserved for I/O R/W (unsupported).

==Test Registers==

{| {{wikitable}}
|-
! Name
! Description
|-
| TR3 - TR5
| Undocumented
|-
| TR6
| Test command register
|-
| TR7
| Test data register
|-
|}

==Protected Mode Registers==

====GDTR====

{| {{wikitable}}
|-
! Bits
! Label
! Description
|-
| 0-15
| Limit
| (Size of [[GDT]]) - 1
|-
| 16-47
| Base
| Starting address of [[GDT]]
|-
|}

Stores the segment selector of the [[GDT]].

====LDTR====

{| {{wikitable}}
! Bits
! Label
! Description
|-
| 0-15
| Limit
| (Size of [[LDT]]) - 1
|-
| 16-47
| Base
| Starting address of [[LDT]]
|-
|}

Stores the segment selector of the [[LDT]].

====IDTR====

{| {{wikitable}}
! Bits
! Label
! Description
|-
| 0-15
| Limit
| (Size of [[IDT]]) - 1
|-
| 16-47
| Base
| Starting address of [[IDT]]
|-
|}

Stores the segment selector of the [[IDT]].

[[Category:X86 CPU]]
[[Category:CPU_Registers]]
````

<a id="w003731"></a>

## W003731 — Porting GCC

Bron-ID: W003731
Namespace: Main (0)
Revisie: 17667 | Datum: 2015-03-02T14:12:23Z
Bron: https://wiki.osdev.org/Porting_GCC
Revisie-URL: https://wiki.osdev.org/index.php?title=Porting_GCC&oldid=17667
Doorverwijzing: Porting GCC to your OS

````text
#REDIRECT [[Porting_GCC_to_your_OS]]
````

<a id="w003732"></a>

## W003732 — Boot Signature

Bron-ID: W003732
Namespace: Main (0)
Revisie: 19564 | Datum: 2016-07-23T11:58:22Z
Bron: https://wiki.osdev.org/Boot_Signature
Revisie-URL: https://wiki.osdev.org/index.php?title=Boot_Signature&oldid=19564
Doorverwijzing: Boot Sequence

````text
#REDIRECT [[Boot Sequence#Master Boot Record]]
````

<a id="w003733"></a>

## W003733 — Null Character

Bron-ID: W003733
Namespace: Main (0)
Revisie: 30226 | Datum: 2026-03-16T14:53:30Z
Bron: https://wiki.osdev.org/Null_Character
Revisie-URL: https://wiki.osdev.org/index.php?title=Null_Character&oldid=30226

````text
The null character, also known as the null terminator, is a character with the value of zero. Besides representing a NOP, nowadays it is known as the control character that indicates the end of an string in [[C]]-like data formats. In essence, the null terminator is a way to encode the end of a string using its own contents. The common alternative to null terminators is storing the length of the string explicitly; this is how languages other than C do it, which is safer and less error prone.

[[File:NullTerminator.png|right|thumb|350px|How the string "osdev" would get encoded in a C-like data format]]

In [[ASCII]], [[Unicode]], and other character sets, it is encoded as a string of ''n'' low bits (or in other words, a representation of the value zero with ''n'' bits). From this point forward, when we mention the null character/terminator, we will refer to a 8-bit ASCII NULL character, represented as 0x00 in hexadecimal or, in C, as the escape sequence '\0' (not to confuse with "\0", which in C is a null-terminated string containing one null character).

It is important to mention that encoding a NULL value to indicate an end is not limited to strings: arrays of ''n'' strings can also have a NULL pointer at it's n-th element to indicate it's end, as <code>argv[]</code> does (the standard way of passing arguments to <code>main()</code> in C).

== How to use and detect a null terminator ==
In C, the null terminator is automatically attached at the end of a string when you create it with double quotation marks ("Hello, OSDev!" is an array of 14 characters, as after the '!', there is a '\0' or null byte). The standard library uses this to detect the length of strings and operate consequentially with them, but behind the header files, the code that detects the null terminator is quite easy but important to understand.
<syntaxhighlight lang="c">
size_t strlen(const char *str)
{
    size_t n = 0;
    
    while (str[n] != '\0') {
        n++;
    }
    
    return n;
}

</syntaxhighlight>
This C code is a quick-and-easy implementation of the <code>strlen()</code> standard function. All it does is increment <code>n</code> while the n-th character of the string <code>str</code> is not a null terminator, and then returns it. If no null byte is found in the string, the loop will keep executing, until it finds a null byte somewhere in memory. If there is memory protection, it may end up reading from an invalid address, generating an exception. Keep in mind that detecting a null terminator is only useful in the context of strings, as memory in general can have arbitrary zeros that don't mean the end of its contents.

== In assembly ==
We talked about the null terminator in C, but not in assembly.
In most of the dialects of assembly, strings variables (or more accurately, labels) can be defined to be null terminated when created. Some assemblers accept the tag <code>.asciiz</code> followed by an string between double quotation marks to create (usually in the data section) a string, that then you can treat as a normal C-like string.
Example under 64 bit Intel assembly, with [[NASM]]:
<syntaxhighlight lang="asm">
bits 64

global main
extern printf

section .data

str:    db  "Hello, OSDev!"         ; NON-NULL-TERMINATED STRING
strnll: db  ", and goodbye!", 0     ; NULL-TERMINATED STRING

section .text

main:
    lea     rdi, [str]
    xor     rax, rax
    call    printf
</syntaxhighlight>
As printf will start reading the characters to print from address "str" until a NULL, the output of the program will be:
<pre>
"Hello, OSDev!, and goodbye!" 
</pre>
== OS design considerations ==
Null terminators are inherently error-prone, given that a missing null character will cause string-handling code to just keep reading memory until it hits a null terminator somewhere. Additionally, it causes some string operations to be more inefficient than they should since the string length is not explicitly recorded, a <code>strlen()</code>-like function must be called every time the string length is needed, which means iterating the entire string.

The proper alternative is to record the string length explicitly. This is how programming languages other than [[C]], specially more modern ones, like [[Zig]] or [[Rust]]. [[Pascal]] for example, records the string length in a single byte before the start of the string data; other languages store it as part of a structure, along with the string data pointer. These strategies can be implemented on C too.

== See also ==
=== External Links ===
* [https://en.wikipedia.org/wiki/Null_character Null character - Wikipedia]
* [https://elixir.bootlin.com/glibc/glibc-2.17/source/string/strlen.c GLibc's strlen() source]
* [https://p403n1x87.github.io/getting-started-with-x86-64-assembly-on-linux.html x64 Intel assembly basics]

[[Category:Theory]]
````

<a id="w003734"></a>

## W003734 — Floppy Programming

Bron-ID: W003734
Namespace: Main (0)
Revisie: 17672 | Datum: 2015-03-02T15:34:48Z
Bron: https://wiki.osdev.org/Floppy_Programming
Revisie-URL: https://wiki.osdev.org/index.php?title=Floppy_Programming&oldid=17672
Doorverwijzing: Floppy Disk Controller

````text
#REDIRECT [[Floppy_Disk_Controller]]
````

<a id="w003735"></a>

## W003735 — Command Line

Bron-ID: W003735
Namespace: Main (0)
Revisie: 29778 | Datum: 2025-12-01T15:48:07Z
Bron: https://wiki.osdev.org/Command_Line
Revisie-URL: https://wiki.osdev.org/index.php?title=Command_Line&oldid=29778

````text
{{Stub}}
The <b>Command Line Interface</b> or <b>CLI</b> is an interface which accepts commands from the user, similar to Windows Command Prompt or Linux Terminal. Check also [[Text UI]].

==Implementation==

To implement a minimal CLI, an operating systems needs at least:
* Access to video memory
* Keyboard driver
* String input, parsing and comparison
* Some functions to execute on commands
Because of their ease of implementation, CLI's are popular in hobby OS projects.

==Advantages==

* Speed, power and control in use
* Resource efficiency
* Scripts to automate tasks (advanced CLI's employ interpreted languages that can be used for both direct command entering and scripting)
* No need for mouse devices and drivers
* Easily compatible with serial lines and TCP sockets, so that a normal command environment can be set up when debugging display drivers or a windowing system
* Copy/paste applies to every program because in the simplest CLI form it's all just text.

Some users with coordination, memory, and attention span issues find command lines much easier than navigating large and inconvenient menu trees, especially if the commands are relatively simple. (Never underestimate the problem-solving skills of people with these issues. ;))

==Disadvantages==

===Serial-related bugs===

Use over a serial line, real or simulated, restricts available key combinations and usually introduces some bugs. Even the newest terminal emulators for POSIX systems operate over a simulated serial line. These bugs stem from 4 causes: in-band signalling, state mismatch, protocol incompatibilities, and unprintable characters. 

State mismatch and in-band signalling issues impact smart command line interfaces ''more'' than 2D TUI because many programs use the same serial line without adequate multiplexing.  See for example bash's behaviour when you start vi, resize the xterm, then quit vi.  bash doesn't know about the changed size.

Protocol mismatch generally concerns the keyboard these days: few ANSI terminals implement the required VT-102 keyboard, but some do.  Output incompatibilities are relatively common in terminal emulators from before about 2003-5, notably XTerm.  Physical terminals may, of course, have bugs in ROM.

All these bugs could be mitigated by a proper terminal driver architecture, but this isn't how things are done in POSIX systems.  Dumb terminals also mitigate the bugs by having no state and few control codes, and generally aren't entirely dumb.  Most support backspace, allowing surprisingly good command-line editing with a bit of software support.

===Usability Issues===

Some users simply have problems getting comfortable with CLIs. Two reasons for this are the need to spell input correctly, and the need to remember commands and their syntax. These reasons overlap: some users find it very hard to remember how to spell unfamiliar words or cryptic abbreviations. It can be hard to remember which specific abbreviation (of several reasonable possibilities) any one program uses. Quick help mitigates these issues, but most command-line programs in modern operating systems have so many features, their quick help exceeds both screen space and attention span of many potential users! It's not all bad, even such large quick help may be copy/pasted, as may text from other documentation. Both copy/paste and tab completion help poor typists enter commands and files. (See also "send"; a combination copy and paste found in some old window systems such as Plan 9's.)

[[Category:Text UI]]
````

<a id="w003736"></a>

## W003736 — CR0

Bron-ID: W003736
Namespace: Main (0)
Revisie: 17970 | Datum: 2015-04-30T08:23:13Z
Bron: https://wiki.osdev.org/CR0
Revisie-URL: https://wiki.osdev.org/index.php?title=CR0&oldid=17970
Doorverwijzing: CPU Registers x86

````text
#REDIRECT[[CPU_Registers_x86#CR0]]
````

<a id="w003737"></a>

## W003737 — Video RAM

Bron-ID: W003737
Namespace: Main (0)
Revisie: 22819 | Datum: 2018-09-10T05:37:55Z
Bron: https://wiki.osdev.org/Video_RAM
Revisie-URL: https://wiki.osdev.org/index.php?title=Video_RAM&oldid=22819

````text
{{Stub}}

Video RAM or VRAM is the common name given to dedicated memory that is located on the graphics hardware. 

Most plug-in video cards have memory on board, which is continuously scanned and outputted over the graphics cable. By detaching it from the system bus, this process does not interrupt the performance of other computation tasks. In some cases{{which}}, this RAM can be used as system memory even though accessing it is significantly slower, but it can also be used as dedicated caching space like the disk cache.

Video RAM is typically exposed through the 128KB at <code>0x000A0000-0x000BFFFF</code>, as well as a large secondary region typically dictated by a corresponding [[PCI]] range. This first block imitates [[VGA Hardware|a VGA]] regarding where the reads and writes appear in the memory, the large region typically maps 1:1 to the actual VRAM on the hardware. If these regions are actually accessible depends on the configuration, and both ways of accessing might require [[How_do_I_set_a_graphics_mode|setting the appropriate video mode]] using the BIOS first - typically [[VBE|a LFB mode]]. A native driver can often enable or disable the entire region without depending on specific video modes, or it can be permanently enabled.

In various low-end systems, graphics hardware actually uses the same memory as the rest of the system, leading to the presence of shared memory. On x86, shared memory is typically represented as a loss in system memory and added dedicated video memory, including fake PCI BARs. On other architectures you might have free choice in which areas of RAM to use for your graphics.
[[Category:Video]]
````

<a id="w003739"></a>

## W003739 — ExFAT

Bron-ID: W003739
Namespace: Main (0)
Revisie: 30517 | Datum: 2026-04-26T04:15:40Z
Bron: https://wiki.osdev.org/ExFAT
Revisie-URL: https://wiki.osdev.org/index.php?title=ExFAT&oldid=30517

````text
{{Filesystems}}
{{In Progress}}

ExFAT, as its name suggests, is an extension of the traditional FAT filesystem. As opposed to older versions, such as FAT16, exFAT has been greatly modernized and has more conveniences from a programming standpoint.

After a long period of keeping the official exFAT spec under their collective hats, Microsoft has finally published it at https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/FileIO/exfat-specification.md on August 27th, 2019. The major structures of the file system are described below.

== Partition Layout ==

{| class="wikitable"
|+ Overview of the order of structures in an exFAT partition
! Section !! Contents !! Offset (sector number) !! Size (in sectors)
|-
| rowspan = 5 | Main Boot Region || [[#Boot_Sector_Structure|Boot Sector]] || 0 || 1
|-
| [[#Extended_Boot_Sectors_Structure|Extended Boot Sectors]] || 1 || 8
|-
| [[#OEM_Parameters_Structure|OEM Parameters]] || 9 || 1
|-
| Reserved sectors || 10 || 1
|-
| [[#Boot_Checksum_Calculation|Boot checksum]] || 11 || 1
|-
| rowspan = 5 | Backup Boot Region || Boot Sector || 12 || 1
|-
| Extended Boot Sectors || 13 || 8
|-
| OEM Parameters || 21 || 1
|-
| Reserved sectors || 22 || 1
|-
| Boot checksum || 23 || 1
|-
| rowspan = 3 | FAT Region || FAT Alignment || 24 || FATOffset - 24
|-
| [[#FAT_Region_Structure|First FAT]] || FATOffset || FATLength
|-
| Second FAT || FATOffset + FATLength || FATLength * (NumberOfFATs - 1)
|-
| rowspan = 3 | Data Region || Cluster Heap Alignment || FATOffset + FATLength * NumberOfFATs || ClusterHeapOffset - (FATOffset + FATLength * NumberOfFATs)
|-
| Cluster Heap || ClusterheapOffset || ClusterCount * 2<sup>SectorsPerClusterShift</sup>
|-
| Excess Space (to end of partition or disk) || ClusterheapOffset + ClusterCount * 2<sup>SectorsPerClusterShift</sup> || VolumeLength - (ClusterHeapOffset + ClusterCount * 2<sup>SectorsPerClusterShift</sup>)
|-
|}

The Backup Boot Region is a duplicate of the Main Boot Region.

The FATAlignment and Cluster Heap Alignment sections appear to be filler to ensure that the FAT and Cluster Heap both start on a good boundary. This has more meaning for removeable media such as flash drives than for hard drives.


== Structures ==

=== Boot Sector Structure ===

{| class="wikitable"
|-
! Offset !! Size !! Name !! Description
|-
| 0x000 || 3 || JumpBoot || Jump instruction to start of boot code. For x86 CPUs, the byte sequence is usually EB 76 90.
|-
| 0x003 || 8 || FileSystemName || The "EXFAT&nbsp;&nbsp;&nbsp;" signature.
|-
| 0x00B || 53 || MustBeZero || This range of bytes is set null to help existing FAT drivers from mistakenly parsing an exFAT partition under the assumption that it contains valid FAT16 or FAT32 data.
|-
| 0x040 || 8 || PartitionOffset || The sector offset of the start of this partition relative to the start of the volume. Can be null.
|-
| 0x048 || 8 || VolumeLength || The total size of this exFAT partition in sectors. Should not be smaller than 1 MiB.
|-
| 0x050 || 4 || FATOffset || Offset in sectors from the start of the partition to the first FAT
|-
| 0x054 || 4 || FATLength || Size of each FAT in sectors
|-
| 0x058 || 4 || ClusterHeapOffset || Offset in sectors from the start of the partition to the start of the "Cluster Heap", known as the "Data Area" on former FAT versions.
|-
| 0x05C || 4 || ClusterCount || The total number of clusters in the Cluster Heap.
|-
| 0x060 || 4 || FirstClusterOfRootDirectory || The starting cluster of the root directory's cluster chain.
|-
| 0x064 || 4 || VolumeSerialNumber || A serial number for this partition, usually comprised by calculating the date and time of format into a single 64-bit value. There is no standard for this calculation; each FAT driver is free to use its own technique.
|-
| 0x068 || 2 || FileSystemRevision || Specifies the version of exFAT specification to which the structures in this partition adhere. For now, there is only a single exFAT specification; version 1.00. However in the future there may be more than this, and a driver should not attempt to parse exFAT structures of an exFAT specification version with which it is not familiar.
|-
| 0x06A || 2 || VolumeFlags || 
{| class="wikitable"
|-
! Bit !! Name !! Description
|-
| 0 || ActiveFAT || Used to implement transaction-safe mechanisms<br>
0 - First FAT and first allocation bitmap are valid<br>
1 - Second FAT and second allocation bitmap are valid
|-
| 1 || VolumeDirty || 0 - Partition is likely in a good state<br>
1 - Partition is likely not in a good state
|-
| 2 || MediaFailure || 0 - No bad clusters aside from those already marked "bad"<br>
1 - An I/O operation failed at some point indicating that the entire partition should be scanned for bad clusters at next mount.
|-
| 3 || ClearToZero || No real meaning... yet!
|-
| 4 - 15 || n/a || Reserved
|-
|}
This word is not included in checksumming.
|-
| 0x06C || 1 || BytesPerSectorShift || The number of places one would need to shift to translate bytes to sectors or sectors to bytes. Serves as an assist to allow shifting instead of multiplication or division.
|-
| 0x06D || 1 || SectorsPerClusterShift || Similar to the above, except used to calculate sectors per cluster.
|-
| 0x06E || 1 || NumberOfFATs || Useful when implementing transaction-safe mechanisms<br>
1 - Partition uses only a single FAT and Allocation Bitmaps<br>
2 - Partition uses two FATs and two Allocation Bitmaps
|-
| 0x06F || 1 || DriveSelect || Similar to the Drive Number field found at offset 0x24 in previous FAT versions. Usually 0x80 for hard disks.
|-
| 0x070 || 1 || PercentInUse || Percent of clusters in the Cluster Heap which are allocated, rounded down to the nearest whole number. This byte is not included in checksumming.
|-
| 0x071 || 7 || Reserved || Do not use
|-
| 0x078 || 390 || BootCode || Boot code pointed to by the initial jump instruction at the beginning of the structure
|-
| 0x1FE || 2 || BootSignature || A series of bytes (0x55, 0xAA) which indicates this is a proper boot sector, similar to previous FAT versions. Should be verified prior to depending on any other fields in the Boot Sector.
|-
|}

=== Extended Boot Sectors Structure ===

{| class="wikitable"
|-
! Offset !! Size !! Name !! Description
|-
| 0x00 || 2<sup>BytesPerSectorShift</sup> - 4 || ExtendedBootCode || Additional space for boot code.
|-
| 2<sup>BytesPerSectorShift</sup> - 4 || 4 || ExtendedBootSignature || A series of bytes (0x00, 0x00, 0x55, 0xAA) which indicates this is a proper boot sector, similar to previous FAT versions, but now a 32-bit value instead of 16-bit. Should be verified prior to depending on any other fields in the Extended Boot Sector. Note that this signature will appear on <i>each</i> of the Extended Boot Sectors.
|-
|}



=== OEM Parameters Structure ===

{| class="wikitable"
|-
! Offset !! Size !! Name !! Description
|-
| 0x000 || 48 || Parameters[0] || rowspan = 10 | A group of parameters in Generic Parameters format.

Each 48-byte entry may be one of:
{| class="wikitable"
|+ NullParameters
|-
! Offset !! Size !! Name !! Description
|-
| 0x00 || 16 || ParametersGUID || The "NullParameters" GUID:<br>
00000000-0000-0000-0000-000000000000
|-
| 0x10 || 32 || CustomDefined || Reserved, do not use
|-
|}

or

{| class="wikitable"
|+ FlashParameters
|-
! Offset !! Size !! Name !! Description
|-
| 0x00 || 16 || ParametersGUID || The "FlashParameters" GUID:<br>
0A0C7E46-3399-4021-90C8-FA6D389C4BA2
|-
| 0x10 || 4 || EraseBlockSize || The size of the flash media's erase block, in bytes
|-
| 0x14 || 4 || PageSize || The page size, in bytes, of the flash media
|-
| 0x18 || 4 || SpareSectors || The number of sectors the flash media has free for use in sparing operations
|-
| 0x1C || 4 || RandomAccessTime || The average number of nanoseconds the flash media needs to randomly access its memory
|-
| 0x20 || 4 || ProgrammingTime || The average number of nanoseconds the flash media needs for a programming operation
|-
| 0x24 || 4 || ReadCycle || The average number of nanoseconds the flash media needs to perform a read cycle
|-
| 0x28 || 4 || WriteCycle || The average number of nanoseconds the flash media needs to perform a write cycle
|-
| 0x2C || 4 || Reserved || Do not use
|-
|}
As of this writing, the exFAT specification only defines the two GUIDs above; additional ones may be defined in future versions.

|-
| 0x030 || 48 || Parameters[1]
|-
| 0x060 || 48 || Parameters[2]
|-
| 0x090 || 48 || Parameters[3]
|-
| 0x0C0 || 48 || Parameters[4]
|-
| 0x0F0 || 48 || Parameters[5]
|-
| 0x120 || 48 || Parameters[6]
|-
| 0x150 || 48 || Parameters[7]
|-
| 0x180 || 48 || Parameters[8]
|-
| 0x1B0 || 48 || Parameters[9]
|-
| 0x1E0 || 2<sup>BytesPerSectorShift</sup> - 480 || Reserved || Do not use
|-
|}



=== FAT Region Structure ===
{| class="wikitable"
|-
! Offset !! Size !! Name !! Description
|-
| 0x00 || 4 || FatEntry[0] || Comprised of a Media Descriptor byte (always 0xF8) followed by three bytes of 0xFF each 
|-
| 0x04 || 4 || FatEntry[1] || Always 0xFFFFFFFF. Do not use, interpret, or change.
|-
| 0x08 || 4 || FatEntry[2] || One of the following:

{| class="wikitable"
|-
! Value !! Description
|-
| 0x00000000 || Denotes a free cluster available for allocation
|-
| 0x00000002 through ClusterCount - 1 || A pointer to the next cluster in the chain
|-
| 0xFFFFFFF7 || Bad cluster
|-
| 0xFFFFFFFF || End-of-chain markeer
|-
|}
These are identical to the values used in previous FAT versions, with the difference that 0x00000001, 0xFFFFFFF0 - 0xFFFFFFF6, and 0xFFFFFFF8 - 0xFFFFFFFE are no longer ascribed any significance.

|-
| ... || ... || ... || ...
|-
|}




== C Code Examples ==

=== Filesystem Header ===

<syntaxhighlight lang="C">
struct bootsector {
	char jump[3];
	char fsid[8];
	char pad[53];
	uint64_t unk; // sectors before first cluster too?
	uint64_t fsSizeInSectors;
        uint32_t sectorsToStartOfFat;
        uint32_t sectorsUsedForFat;
        uint32_t sectorStartOfFirstCluster;
        uint32_t clusterCountInFs;
        uint32_t clusterForRootDirectory;
        uint32_t unk[2];
        uint8_t logBytesPerSector, logSectorsPerCluster, fatCount, driveId; // 2-log for both. bytesPerCluster = 1 << (logBytesPerSector + logSectorsPerCluster).
        uint8_t percentInUse; // for display ala windows computer explorer
        uint8_t unk[7];
        char bootcode[390];
        char bootsign[2];
};
</syntaxhighlight>
Block bitmap is cluster 2, upcase tbl is cluster 3. 


=== Boot Checksum Calculation ===
<syntaxhighlight lang="C">
UInt32 BootChecksum (UCHAR * Sectors, USHORT BytesPerSector)
{
  /*
    UCHAR * Sectors, above, points to an in-memory copy of the 11 sectors
    That is, 0 - 10 for the Main Boot Region, or 12 - 22 for the Backup Boot Region
  */

  UInt32 NumberOfBytes = (UInt32)BytesPerSector * 11;
  UInt32 Checksum = 0;
  UInt32 Index;

  for (Index = 0; Index < NumberOfBytes; Index++)
  {
    if ((Index == 106) || (Index == 107) || (Index == 112))
    {
      continue;
    }
    Checksum = ((Checksum&1) ? 0x80000000 : 0) + (Checksum>>1) + (UInt32)Sectors[Index];
  }
  return Checksum;
}
</syntaxhighlight>



=== Directory entries ===

32-byte entries. Entries contain one of a few possible entry types, first byte identifies which type it is. The second byte identifies how many subsequent entries are part of this entry. All bytes after are dependent on the type of entry. Any entry with the highest bit of entrytype set is not used.

<syntaxhighlight lang="c">
struct {
        uint8_t entrytype = 0x85;
        uint8_t entrycount; // This is the number of subsequent entries that also belong to this "file" entry. Should be at least 2, one 0xC0 and one 0xC1 for info and filename.
        uint16_t checksum;
        uint16_t flags; // 0x10 == directory
        uint16_t rsvd; // Reserved
        uint32_t creation, modification, access;
        char pad[12];
} fileEntry;
</syntaxhighlight>

<syntaxhighlight lang="c">
struct {
        uint8_t entryType = 0xC0;
        uint8_t flags;
        uint8_t reserved;
        uint8_t filenameLengthInBytes;
        uint16_t filenameHash;
        uint16_t reserved_2;
        uint64_t valid_filesize;
        uint32_t reserved3;
        uint32_t startCluster; // minus 2! Same as fat12/16/32.
        uint64_t filesize;
} fileInfoEntry;
</syntaxhighlight>

<syntaxhighlight lang="C">
struct {
        uint8_t entrytype = 0xC1, entrycount = 0x00;
        uint16_t name[15];
} filenameEntry;
</syntaxhighlight>



== Disk Hexdump Examples ==

=== Short filenames ===
    01040300  85 02 c8 f9 10 00 00 00  5b a9 47 45 6f a9 47 45  |........[.GEo.GE|
    01040310  6f a9 47 45 25 25 88 88  88 00 00 00 00 00 00 00  |o.GE%%..........|
    01040320  c0 03 00 05 ae 26 00 00  00 00 02 00 00 00 00 00  |.....&..........|
    01040330  00 00 00 00 17 00 00 00  00 00 02 00 00 00 00 00  |................|
    01040340  c1 00 69 00 6d 00 61 00  67 00 65 00 00 00 00 00  |..i.m.a.g.e.....|
    01040350  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|

=== Longer filenames ===
    01080060  85 03 34 aa 10 00 00 00  1c a5 47 45 92 0a 48 45  |..4.......GE..HE|
    01080070  92 0a 48 45 94 94 88 88  88 00 00 00 00 00 00 00  |..HE............|
    01080080  c0 03 00 18 23 20 00 00  00 00 02 00 00 00 00 00  |....# ..........|
    01080090  00 00 00 00 07 00 00 00  00 00 02 00 00 00 00 00  |................|
    010800a0  c1 00 63 00 6f 00 6d 00  2e 00 67 00 6f 00 6f 00  |..c.o.m...g.o.o.|
    010800b0  67 00 6c 00 65 00 2e 00  61 00 6e 00 64 00 72 00  |g.l.e...a.n.d.r.|
    010800c0  c1 00 6f 00 69 00 64 00  2e 00 6d 00 75 00 73 00  |..o.i.d...m.u.s.|
    010800d0  69 00 63 00 00 00 00 00  00 00 00 00 00 00 00 00  |i.c.............|

=== Really Long filenames ===
    01100640  85 05 a9 6f 20 00 00 00  25 38 48 45 26 38 48 45  |...o ...%8HE&8HE|
    01100650  25 38 48 45 64 64 00 00  00 00 00 00 00 00 00 00  |%8HEdd..........|
    01100660  c0 01 00 32 c6 a5 00 00  d8 52 76 00 00 00 00 00  |...2.....Rv.....|
    01100670  00 00 00 00 14 46 00 00  d8 52 76 00 00 00 00 00  |.....F...Rv.....|
    01100680  c1 00 30 00 30 00 33 00  20 00 2d 00 20 00 4c 00  |..0.0.3. .-. .L.|
    01100690  65 00 64 00 20 00 5a 00  65 00 70 00 70 00 65 00  |e.d. .Z.e.p.p.e.|
    011006a0  c1 00 6c 00 69 00 6e 00  20 00 2d 00 20 00 53 00  |..l.i.n. .-. .S.|
    011006b0  74 00 61 00 69 00 72 00  77 00 61 00 79 00 20 00  |t.a.i.r.w.a.y. .|
    011006c0  c1 00 74 00 6f 00 20 00  68 00 65 00 61 00 76 00  |..t.o. .h.e.a.v.|
    011006d0  65 00 6e 00 20 00 2d 00  20 00 31 00 39 00 37 00  |e.n. .-. .1.9.7.|
    011006e0  c1 00 32 00 2e 00 6d 00  70 00 33 00 00 00 00 00  |..2...m.p.3.....|    
    011006f0  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|

[[Category:Filesystems]]
````

<a id="w003740"></a>

## W003740 — User Interface

Bron-ID: W003740
Namespace: Main (0)
Revisie: 22820 | Datum: 2018-09-10T05:39:40Z
Bron: https://wiki.osdev.org/User_Interface
Revisie-URL: https://wiki.osdev.org/index.php?title=User_Interface&oldid=22820

````text
{{Stub}}
The <b>User Interface</b> or <b>UI</b> is a subcategory of the human-machine interface and plays an important role in an Operating System project. The term "UI" describes the interaction of the OS or rather of applications with the user. The other direction, the interaction of the user with the OS or rather application is another subcategory of the human-machine interface. Nowadays this interface is visual, also some OS's support haptic, e.g. vibration, or auditory, f.e. for notifications, feedback. This interface will form the first impression of the user and based on it the user will decide either to use the corresponding OS or switch to another one.<br>
There are two main types of UI's: [[Text UI|text user interfaces]], often called [[Command Line|command line interfaces]], and [[GUI|graphical user interfaces]]. Text interfaces are easier to implement, while graphical user interfaces are easier to use, especially for beginners due to not needing to remember commands.
[[Category:UI]]
````

<a id="w003744"></a>

## W003744 — Endianness

Bron-ID: W003744
Namespace: Main (0)
Revisie: 29914 | Datum: 2026-01-22T04:02:03Z
Bron: https://wiki.osdev.org/Endianness
Revisie-URL: https://wiki.osdev.org/index.php?title=Endianness&oldid=29914

````text
Endianness refers to the order in which bytes of a multi-byte structure are placed to main memory (RAM).
The <b>bit</b> order in any byte is normally from bit seven down to bit zero (little-endian) and remains preserved.

==Big- vs. little-endian==

Big-endian stores a value in order, little-endian in reversed order of significance. In big-endian the most-significant (left-hand) byte, in little-endian the least-significant (right-hand) byte is stored at the lowest address.<br>
The following table shows the byte number of a 32-bit structure with byte three equals bit 31 down to 24, byte two equals bit 23 down to 16, byte one equals bit 15 down to 8 and byte zero equals bit seven down to zero.

{| {{wikitable}}
|- style="text-align:center" style="background: #FFFFFF"
! Memory<br>address
| style="background: #EEEEEE" |
! <font color="white">' </font> n <font color="white">'</font>
! n+1
! n+2
! n+3
|-
| colspan=7 style="background: #EEEEEE" |
|- style="text-align:center"
! Big-endian:
| style="background: #EEEEEE" |
| 3
| 2
| 1
| 0
|-
| colspan=7 style="background: #EEEEEE" |
|- style="text-align:center; background: #FFFFFF"
! Little-endian:

[[Category: Theory]]
| style="background: #EEEEEE" |
| 0
| 1
| 2
| 3
|-
|}

Little-endian may seem illogical until one considers that a variable can be used more easily as 8-bit, 16-bit, 32-bit or even 64-bit value without changing its base address. Therefore, AMD and Intel use little-endian order.

Big-endian is sometimes referred to as <b>network byte order</b>. Big endian order is used f.e. in natural languages or .png-files.

==Examples==

*Table
{| {{wikitable}}
|- style="text-align:center" style="background: #FFFFFF"
! colspan=2 | Memory<br>address
! n
! n+1
! n+2
! ...
|-
| colspan=7 style="background: #EEEEEE" |
|- style="text-align:center"
! rowspan=2 style="background: #F5F5F5" | Value =<br> 0x15
! style="background: #FFFFE0" | Big-endian:
| style="background: #FFFFE0" | 0x15
| style="background: #FFFFE0" | -
| style="background: #FFFFE0" | -
| style="background: #FFFFE0" | -
|- style="text-align:center; background: #FFFFFF"
! Little-endian:
| 0x15
| -
| -
| -
|-
| colspan=7 style="background: #EEEEEE" |
|- style="text-align:center"
! rowspan=2 style="background: #F5F5F5" | Value =<br> 0x0A15
! style="background: #FFFFE0" | Big-endian:
| style="background: #FFFFE0" | 0x0A
| style="background: #FFFFE0" | 0x15
| style="background: #FFFFE0" | -
| style="background: #FFFFE0" | -
|- style="text-align:center; background: #FFFFFF"
! Little-endian:
| 0x15
| 0x0A
| -
| -
|-
| colspan=7 style="background: #EEEEEE" |
|- style="text-align:center"
! rowspan=2 style="background: #F5F5F5" | Value =<br> 0x780A15
! style="background: #FFFFE0" | Big-endian:
| style="background: #FFFFE0" | 0x78
| style="background: #FFFFE0" | 0x0A
| style="background: #FFFFE0" | 0x15
| style="background: #FFFFE0" | -
|- style="text-align:center; background: #FFFFFF"
! Little-endian:
| 0x15
| 0x0A
| 0x78
| -
|-
| colspan=7 style="background: #EEEEEE" |
|-
| colspan=7 style="text-align:center;background: #FFFFFF" | a.s.o.
|-
|}

*Let's assume a x86-computer, which means little-endian order. Register eax contains the value 0xCAFEF00D. You store this register at address 0:

Memmory address: 0 contains: 0x0D<br>
Memmory address: 1 contains: 0xF0<br>
Memmory address: 2 contains: 0xFE<br>
Memmory address: 3 contains: 0xCA

Now you read into register ax the value from address 0:

Register ax contains: 0xF00D

*Let's assume big-endian order. A 32-bit register contains the value 0xCAFEF00D. You store this register at address 0:

Memmory address: 0 contains: 0xCA<br>
Memmory address: 1 contains: 0xFE<br>
Memmory address: 2 contains: 0xF0<br>
Memmory address: 3 contains: 0x0D

Now you read into a 16-bit register the value from address 0:

This 16-bit register contains: 0xCAFE<br>
(Which is the upper-half of the value. You probably wanted the lower-half and therefor must change the base address and read from address: 2.)

*578 Pound Sterling<br>
"byte" from 9 to 0:<br>
- big-endian: 578 £<br>
- little-endian: 875 £<br>
"byte" from 99 to 0:<br>
- big-endian: 578 £<br>
- little-endian: 7805 £

==Common CPUs==

{| {{wikitable}}
|-
! CPU
! Endianness
|-
| x86
| little
|-
| x86-64
| little
|-
| MIPS
| both
|-
| Motorola
| big
|-
| 68k
| big
|-
| PowerPC
| both
|-
|}
````

<a id="w003745"></a>

## W003745 — Little-endian

Bron-ID: W003745
Namespace: Main (0)
Revisie: 17960 | Datum: 2015-04-29T19:03:01Z
Bron: https://wiki.osdev.org/Little-endian
Revisie-URL: https://wiki.osdev.org/index.php?title=Little-endian&oldid=17960
Doorverwijzing: Endianness

````text
#REDIRECT [[Endianness]]
````

<a id="w003746"></a>

## W003746 — Big-endian

Bron-ID: W003746
Namespace: Main (0)
Revisie: 17961 | Datum: 2015-04-29T19:03:38Z
Bron: https://wiki.osdev.org/Big-endian
Revisie-URL: https://wiki.osdev.org/index.php?title=Big-endian&oldid=17961
Doorverwijzing: Endianness

````text
#REDIRECT [[Endianness]]
````

<a id="w003747"></a>

## W003747 — Editors

Bron-ID: W003747
Namespace: Main (0)
Revisie: 17740 | Datum: 2015-03-04T15:16:41Z
Bron: https://wiki.osdev.org/Editors
Revisie-URL: https://wiki.osdev.org/index.php?title=Editors&oldid=17740
Doorverwijzing: List of editors

````text
#REDIRECT [[List_of_editors]]
````

<a id="w003748"></a>

## W003748 — Symbolic link

Bron-ID: W003748
Namespace: Main (0)
Revisie: 28100 | Datum: 2023-07-09T22:36:58Z
Bron: https://wiki.osdev.org/Symbolic_link
Revisie-URL: https://wiki.osdev.org/index.php?title=Symbolic_link&oldid=28100

````text
{{Filesystems}}

<!-- {{In Progress}} -->
==Introduction==
A symbolic link is a special type of file that refers to another file or directory and is most often encountered in UNIX derived/inspired operating systems and their file systems.
Symbolic links are real files that contain a path which is automatically resolved when the link is accessed.

===Path resolution===

There are multiple ways of handling symbolic links during path resolution: Some operating systems maintain the hierarchy of the path leading to the symlink while others substitute the entire hierarchy of the path for that of the target. UNIX-like operating systems handle this in a strange way: The kernel and C library do the latter while some shells maintain a history of path changes and resolve the parent of a symlinked directory as the parent of the symlink.

Example:
On a system substituting the hierarchy (Linux, glibc, shown as shell commands for clarity) :
<code><pre>
$ ln -T /dir1/sdir1 /dir2/sdirlink1
$ cd /dir2/sdirlink1
$ pwd
/dir1/sdir1
$ cd ..
$ pwd
/dir1
$ cd /
$ cd /dir2/sdirlink1/..
$ pwd
/dir1
</pre></code>

On a system maintaining the symlink's hierarchy (posnk (old version), newlib, shown as shell commands for clarity) :
<code><pre>
$ ln -T /dir1/sdir1 /dir2/sdirlink1
$ cd /dir2/sdirlink1
$ pwd
/dir2/sdirlink1
$ cd ..
$ pwd
/dir2
$ cd /
$ cd /dir2/sdirlink1/..
$ pwd
/dir2
</pre></code>

[[Category:Filesystems]]
````

<a id="w003749"></a>

## W003749 — Shared library

Bron-ID: W003749
Namespace: Main (0)
Revisie: 28703 | Datum: 2024-03-16T16:41:53Z
Bron: https://wiki.osdev.org/Shared_library
Revisie-URL: https://wiki.osdev.org/index.php?title=Shared_library&oldid=28703

````text
{{Stub}}
A shared library is a file which is shared between executables.

==Usefulness==

Some programs may contain identical parts. In the case of a static library, these parts have their own copy in each executable and program size is immense. This means it uses more memory and more disk space. In the case of a shared library, these parts are in a central file, all those unnecessary copies are eliminated and only one is kept. This approach uses less disk space and less memory. This is a bit similar to system calls located only in the kernel file, but used by all other executables. Shared libraries, as all libraries, can contain functions, structures, configuration data and values.

At least, that's the idea. In practice, systems with shared libraries may use more memory and/or disk space. When an OS has demand paging, (loading parts of files on demand,) shared libraries may in some cases use more memory than static. Programs using shared libraries and shared libraries themselves must be re-linked at load time. This means the in-memory copy is not the same as the on-disk copy, and so demand paging cannot be used in the same way. The relinked code could be paged out to swap files, (using more disk space!) but first it has to all be loaded for re-linking. This contrasts to a statically-linked executable, of which only the parts which are used need be loaded. For instance, bash contains code for command-line editing. This code is not used when running scripts, but for every script run, the editing code must be loaded and linked against the readline library.

==Implementation==

TODO
````

<a id="w003750"></a>

## W003750 — Select

Bron-ID: W003750
Namespace: Main (0)
Revisie: 17763 | Datum: 2015-03-08T09:18:24Z
Bron: https://wiki.osdev.org/Select
Revisie-URL: https://wiki.osdev.org/index.php?title=Select&oldid=17763

````text

In many cases, applications will want to communicate with other processes on many file descriptors at once, waiting for messages from many peers (these descriptors can be many things such as pipes, sockets or terminal input). This is typically the case of server software, but many applications that handle complex interactions end up having to work this way. A first solution to do this is to use one thread per communication channel, and let all the threads communicate via shared memory.

Another solution is provided in most unix implementations : the <tt>select</tt> system call. This system call allows a single thread to wait on multiple file descriptors for one to become ready for reading/writing. Therefore the core of a server program becomes a single thread that selects a peer to communicate with, reads its message, processes it, and loops around.

==Advantages & inconvenients==

There are several advantages to this method, in particular the fact that you don't need to allocate as many threads as you have clients, and the fact that the synchronization becomes much simpler (it actually is the kernel that does the synchronization, not the process).

The main inconvenient is that while the server is processing one messages, all the other clients must wait for their request to be processed. An intermediate solution could be designed, where the server software uses a fixed number of threads, for instance one that reads the requests and several that process them.

==Implementation==

An implementation of the select system call basically does the following :

# Look if any of the descriptors is ready for reading/writing
# If so, return it
# Otherwise, wait on all descriptor objects
# When resumed, restart at 1.

This implementation supposes that your kernel has a sort of "wait_on_objects" function that pauses the current thread and waits for a "ready" event on any of the objects (basically the "ready" event for a pipe or a socket happens when the other end has written something and the buffer is no longer empty).

==Timeout==

On recent unixes the select system calls accept a timeout argument, that causes the select system call to end even if no descriptor is ready once the timeout has passed.

==See Also==

*[[Unix Pipes]], [[Socket|Sockets]]
*[[Eleanore Semaphore]]

===External links===
*[[wikipedia:select_(Unix)|The Wikipedia page on the select syscall]]

[[Category:IPC]]
[[Category:Networking]]
````

<a id="w003751"></a>

## W003751 — Socket

Bron-ID: W003751
Namespace: Main (0)
Revisie: 17765 | Datum: 2015-03-08T09:27:30Z
Bron: https://wiki.osdev.org/Socket
Revisie-URL: https://wiki.osdev.org/index.php?title=Socket&oldid=17765

````text
A socket is basically a way for a server program to communicate with many clients. Sockets can have several underlying protocols : TCP/IP sockets enable
communication on the network, Unix sockets enable communication on a local machine.

Once a connection is established between the server and the client, both process get a new file descriptor where they can write to send a message to the other process and read for messages sent by the other process. On a local machine, this is mostly like having two [[Unix Pipes|pipes]], one going in each way, and sharing the same file descriptor number.

The server socket is the part that accepts new connections. It is also a file descriptor in the server's process, but does not handle usual read/write requests. Instead it handles an "accept" syscall that basically waits for a new connection and when one arrives establishes the connection and returns the file descriptor number for that connection. The clients that wish to establish connections must issue a connect call on the server socket, which is identified either by a file name (for unix sockets) or by a network address.

There are several solutions for server software to handle communication with multiple peers at once. A first solution is to use multiple threads. Another solution available on most unix implementations is to use the [[Select|select]] system call.


[[Category:IPC]]
[[Category:Networking]]
````

<a id="w003752"></a>

## W003752 — PowerPC

Bron-ID: W003752
Namespace: Main (0)
Revisie: 17774 | Datum: 2015-03-11T15:18:32Z
Bron: https://wiki.osdev.org/PowerPC
Revisie-URL: https://wiki.osdev.org/index.php?title=PowerPC&oldid=17774
Doorverwijzing: Category:PowerPC

````text
#REDIRECT [[Category:PowerPC]]
````

<a id="w003753"></a>

## W003753 — Bootloaders

Bron-ID: W003753
Namespace: Main (0)
Revisie: 17775 | Datum: 2015-03-11T15:21:10Z
Bron: https://wiki.osdev.org/Bootloaders
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootloaders&oldid=17775
Doorverwijzing: Category:Bootloaders

````text
#REDIRECT [[Category:Bootloaders]]
````

<a id="w003754"></a>

## W003754 — Assemblers

Bron-ID: W003754
Namespace: Main (0)
Revisie: 17776 | Datum: 2015-03-11T15:23:22Z
Bron: https://wiki.osdev.org/Assemblers
Revisie-URL: https://wiki.osdev.org/index.php?title=Assemblers&oldid=17776
Doorverwijzing: Category:Assemblers

````text
#REDIRECT [[Category:Assemblers]]
````

<a id="w003755"></a>

## W003755 — User:Exetwezz

Bron-ID: W003755
Namespace: User (2)
Revisie: 30612 | Datum: 2026-05-23T11:39:58Z
Bron: https://wiki.osdev.org/User:Exetwezz
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AExetwezz&oldid=30612

````text

````

<a id="w003756"></a>

## W003756 — Compilers

Bron-ID: W003756
Namespace: Main (0)
Revisie: 17780 | Datum: 2015-03-11T18:23:35Z
Bron: https://wiki.osdev.org/Compilers
Revisie-URL: https://wiki.osdev.org/index.php?title=Compilers&oldid=17780
Doorverwijzing: Category:Compilers

````text
#REDIRECT [[Category:Compilers]]
````

<a id="w003757"></a>

## W003757 — CR4

Bron-ID: W003757
Namespace: Main (0)
Revisie: 17971 | Datum: 2015-04-30T08:23:24Z
Bron: https://wiki.osdev.org/CR4
Revisie-URL: https://wiki.osdev.org/index.php?title=CR4&oldid=17971
Doorverwijzing: CPU Registers x86

````text
#REDIRECT[[CPU_Registers_x86#CR4]]
````

<a id="w003759"></a>

## W003759 — Talk:Kernel Stdio Theory

Bron-ID: W003759
Namespace: Talk (1)
Revisie: 17812 | Datum: 2015-03-17T08:15:58Z
Bron: https://wiki.osdev.org/Talk:Kernel_Stdio_Theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AKernel_Stdio_Theory&oldid=17812

````text
We should try to add some links to other papers/places that talk about the theory, or link to some actual implementations if possible. -r0nk
````

<a id="w003760"></a>

## W003760 — File:Vector shell.png

Bron-ID: W003760
Namespace: File (6)
Revisie: 17848 | Datum: 2015-04-02T21:26:15Z
Bron: https://wiki.osdev.org/File:Vector_shell.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVector_shell.png&oldid=17848

````text
Screenshot of the Vector shell demonstrating the CPUID driver and the disk password.
````

<a id="w003761"></a>

## W003761 — USB Flash Memory

Bron-ID: W003761
Namespace: Main (0)
Revisie: 28092 | Datum: 2023-07-09T21:27:51Z
Bron: https://wiki.osdev.org/USB_Flash_Memory
Revisie-URL: https://wiki.osdev.org/index.php?title=USB_Flash_Memory&oldid=28092

````text
People can save and move data from one computer to another using USB flash disks. If you have a USB port and want to access USB flash disks, then you need a [[USB]] driver.

== For the [[EHCI]] controller ==

EHCI is probably the most supported USB controller in hobby operating systems.

==== Initialize the controller ====

After finding the controller, you have to initialize it. Implementations may vary from OS to OS, but the basic line is the same.
* Enable Bus Master and Memory Space
* Save Base Address
* Save Operational Base Address
* Save Extended Capabilities Pointer
* Check if Extended Capabilities Pointer < 40; if true, skip this step
** Check if BIOS owns the controller; if true, get ownership
* Stop and reset the controller (but '''not''' start)
* Setup Queue Head
* Start the controller

==== Find devices ====

* Scan USB ports for connected devices; if false, go to next port
* Check if low speed; if true, go to next port (mass-storage devices are hi-speed)
* Reset and enable the port which contains the device.

==== Prepare devices ====

* Get device-configuration-interface-endpoint descriptors.
** Check if the device is mass-storage class
** Check if the device responds to [[SCSI]] commands
** Check if the protocol is Bulk-Only
** If all above true, proceed
* Assign unique USB address and use it from now
* Assign device configuration (usually got from configuration descriptor)
* Get and use endpoints from endpoint descriptor
* Send Inquiry Command
* Send Test Unit Ready Command
* Send Read Capacity Command

==== Read/write ====

Only after those steps, request read and write commands.

== See Also ==

==== External Links ====
* [https://code.google.com/p/tatos/ TatOS website] — a hobby operating system the main feature of which is a driver for USB flash drives.

[[Category:USB]]
[[Category:Storage]]
````

<a id="w003762"></a>

## W003762 — Boot signature

Bron-ID: W003762
Namespace: Main (0)
Revisie: 20682 | Datum: 2017-04-10T17:39:51Z
Bron: https://wiki.osdev.org/Boot_signature
Revisie-URL: https://wiki.osdev.org/index.php?title=Boot_signature&oldid=20682
Doorverwijzing: Boot Sequence

````text
#REDIRECT [[Boot Sequence#Master Boot Record]]
````

<a id="w003764"></a>

## W003764 — Null character

Bron-ID: W003764
Namespace: Main (0)
Revisie: 17718 | Datum: 2015-03-03T19:52:11Z
Bron: https://wiki.osdev.org/Null_character
Revisie-URL: https://wiki.osdev.org/index.php?title=Null_character&oldid=17718
Doorverwijzing: Null Character

````text
#REDIRECT [[Null Character]]
````

<a id="w003766"></a>

## W003766 — Linux

Bron-ID: W003766
Namespace: Main (0)
Revisie: 17876 | Datum: 2015-04-12T10:48:52Z
Bron: https://wiki.osdev.org/Linux
Revisie-URL: https://wiki.osdev.org/index.php?title=Linux&oldid=17876
Doorverwijzing: Linux Kernel Primer

````text
#REDIRECT [[Linux Kernel Primer]]
````

<a id="w003767"></a>

## W003767 — User:KemyLand

Bron-ID: W003767
Namespace: User (2)
Revisie: 17879 | Datum: 2015-04-12T22:11:39Z
Bron: https://wiki.osdev.org/User:KemyLand
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AKemyLand&oldid=17879

````text
My editing room :)

=== Internet Protocol ===

This article talks about the IPv4 (Internet Protocol, version 4). IPv4, as defined in [https://www.ietf.org/rfc/rfc791.txt RFC 791] in September 1981, is used to communicate devices through the Internet, i.e: the world-wide computer network you're using right now. This is done with the concept of an IP address, that is, a 32-bit number uniquely identifying a device or network in the Internet. [[ARP]] runs on the link-layer, so it's irrelevant for this article.

== Relation to other protocols ==
The IP is designed to be used in a protocol-stack manner. In the first layer you'll find [[ARP]] and lots of strange stuffs. On the second layer you'll find IPv4 and [[ICMP]]. On the third layer you find either [[TCP]] or [[UDP]], and the list goes on.

Each stack layer uses the abstractions provided by lower-layers, to provide abstractions for higher-layers.

== The IPv4 Header ==
The IP Header is actually the unique data-structure defined by the IP protocol. It appears like follows. Note that bit-fields smaller than 8-bits may be used (not available in C), as well as non-2-power fields:
<source lang="C">
typedef UInt32 IPAddr;

struct IPv4Header {
  UInt4  version;
  UInt4  headerLen;
  UInt8  service;
  UInt16 datagramLen;
  UInt16 id;
  UInt3  flags;
  UInt13 fragmentOff;
  UInt8  lifeTime;
  UInt8  protocol;
  UInt16 checksum;
  IPAddr srcAddr;
  IPAddr destAddr;
  UInt8  options[?];
  UInt8  padding[?];
};
</source>
````

<a id="w003768"></a>

## W003768 — Intel Ethernet i217

Bron-ID: W003768
Namespace: Main (0)
Revisie: 27712 | Datum: 2023-03-30T20:11:25Z
Bron: https://wiki.osdev.org/Intel_Ethernet_i217
Revisie-URL: https://wiki.osdev.org/index.php?title=Intel_Ethernet_i217&oldid=27712

````text
{{FirstPerson}}
== Network Driver for Intel Ethernet Cards I217 and 82577LM ==

I am writing this Wiki as a demonstration of my own experience of getting a working driver for the Intel I217 and 82577LM network cards to work, on a real native bare metal hardware, namely Thinkpads W540 and W510. Linux uses the e1000e network driver for those cards. I have started from a working e1000 driver that I have developed for my OS and which is operational on Qemu, Bochs, and VirtualBox. The main objective of this Wiki is to try to highlight the differences and the addition needed on an operations e1000 driver to work handle network cards that work with the e1000e. So the provided knowledge in this wiki might be applicable on other Intel interfaces. For completion, I will present in this wiki my e1000 driver with the additions that made it work on those native NICs, I217 and 82577LM. I built my original e1000 driver based on information from OSDev and some hobby operating systems on github.

It is very important to highlight that this wiki does not utilize all the features in the above NICs, but it show how to configure the NICs for basic functionality such as initialization, read packets, and write packets. 

== Card Addresses and Data Structures ==

To start with, lets state some macro definitions that we are going to use in the code.
<source lang="c">

#define INTEL_VEND     0x8086  // Vendor ID for Intel 
#define E1000_DEV      0x100E  // Device ID for the e1000 Qemu, Bochs, and VirtualBox emmulated NICs
#define E1000_I217     0x153A  // Device ID for Intel I217
#define E1000_82577LM  0x10EA  // Device ID for Intel 82577LM


// I have gathered those from different Hobby online operating systems instead of getting them one by one from the manual

#define REG_CTRL        0x0000
#define REG_STATUS      0x0008
#define REG_EEPROM      0x0014
#define REG_CTRL_EXT    0x0018
#define REG_IMASK       0x00D0
#define REG_RCTRL       0x0100
#define REG_RXDESCLO    0x2800
#define REG_RXDESCHI    0x2804
#define REG_RXDESCLEN   0x2808
#define REG_RXDESCHEAD  0x2810
#define REG_RXDESCTAIL  0x2818

#define REG_TCTRL       0x0400
#define REG_TXDESCLO    0x3800
#define REG_TXDESCHI    0x3804
#define REG_TXDESCLEN   0x3808
#define REG_TXDESCHEAD  0x3810
#define REG_TXDESCTAIL  0x3818


#define REG_RDTR         0x2820 // RX Delay Timer Register
#define REG_RXDCTL       0x2828 // RX Descriptor Control
#define REG_RADV         0x282C // RX Int. Absolute Delay Timer
#define REG_RSRPD        0x2C00 // RX Small Packet Detect Interrupt



#define REG_TIPG         0x0410      // Transmit Inter Packet Gap
#define ECTRL_SLU        0x40        //set link up


#define RCTL_EN                         (1 << 1)    // Receiver Enable
#define RCTL_SBP                        (1 << 2)    // Store Bad Packets
#define RCTL_UPE                        (1 << 3)    // Unicast Promiscuous Enabled
#define RCTL_MPE                        (1 << 4)    // Multicast Promiscuous Enabled
#define RCTL_LPE                        (1 << 5)    // Long Packet Reception Enable
#define RCTL_LBM_NONE                   (0 << 6)    // No Loopback
#define RCTL_LBM_PHY                    (3 << 6)    // PHY or external SerDesc loopback
#define RTCL_RDMTS_HALF                 (0 << 8)    // Free Buffer Threshold is 1/2 of RDLEN
#define RTCL_RDMTS_QUARTER              (1 << 8)    // Free Buffer Threshold is 1/4 of RDLEN
#define RTCL_RDMTS_EIGHTH               (2 << 8)    // Free Buffer Threshold is 1/8 of RDLEN
#define RCTL_MO_36                      (0 << 12)   // Multicast Offset - bits 47:36
#define RCTL_MO_35                      (1 << 12)   // Multicast Offset - bits 46:35
#define RCTL_MO_34                      (2 << 12)   // Multicast Offset - bits 45:34
#define RCTL_MO_32                      (3 << 12)   // Multicast Offset - bits 43:32
#define RCTL_BAM                        (1 << 15)   // Broadcast Accept Mode
#define RCTL_VFE                        (1 << 18)   // VLAN Filter Enable
#define RCTL_CFIEN                      (1 << 19)   // Canonical Form Indicator Enable
#define RCTL_CFI                        (1 << 20)   // Canonical Form Indicator Bit Value
#define RCTL_DPF                        (1 << 22)   // Discard Pause Frames
#define RCTL_PMCF                       (1 << 23)   // Pass MAC Control Frames
#define RCTL_SECRC                      (1 << 26)   // Strip Ethernet CRC

// Buffer Sizes
#define RCTL_BSIZE_256                  (3 << 16)
#define RCTL_BSIZE_512                  (2 << 16)
#define RCTL_BSIZE_1024                 (1 << 16)
#define RCTL_BSIZE_2048                 (0 << 16)
#define RCTL_BSIZE_4096                 ((3 << 16) | (1 << 25))
#define RCTL_BSIZE_8192                 ((2 << 16) | (1 << 25))
#define RCTL_BSIZE_16384                ((1 << 16) | (1 << 25))


// Transmit Command

#define CMD_EOP                         (1 << 0)    // End of Packet
#define CMD_IFCS                        (1 << 1)    // Insert FCS
#define CMD_IC                          (1 << 2)    // Insert Checksum
#define CMD_RS                          (1 << 3)    // Report Status
#define CMD_RPS                         (1 << 4)    // Report Packet Sent
#define CMD_VLE                         (1 << 6)    // VLAN Packet Enable
#define CMD_IDE                         (1 << 7)    // Interrupt Delay Enable


// TCTL Register

#define TCTL_EN                         (1 << 1)    // Transmit Enable
#define TCTL_PSP                        (1 << 3)    // Pad Short Packets
#define TCTL_CT_SHIFT                   4           // Collision Threshold
#define TCTL_COLD_SHIFT                 12          // Collision Distance
#define TCTL_SWXOFF                     (1 << 22)   // Software XOFF Transmission
#define TCTL_RTLC                       (1 << 24)   // Re-transmit on Late Collision

#define TSTA_DD                         (1 << 0)    // Descriptor Done
#define TSTA_EC                         (1 << 1)    // Excess Collisions
#define TSTA_LC                         (1 << 2)    // Late Collision
#define LSTA_TU                         (1 << 3)    // Transmit Underrun
</source>

Now lets define the data structures for the transmit and receive buffers

<source lang="c">

#define E1000_NUM_RX_DESC 32
#define E1000_NUM_TX_DESC 8

struct e1000_rx_desc {
        volatile uint64_t addr;
        volatile uint16_t length;
        volatile uint16_t checksum;
        volatile uint8_t status;
        volatile uint8_t errors;
        volatile uint16_t special;
} __attribute__((packed));

struct e1000_tx_desc {
        volatile uint64_t addr;
        volatile uint16_t length;
        volatile uint8_t cso;
        volatile uint8_t cmd;
        volatile uint8_t status;
        volatile uint8_t css;
        volatile uint16_t special;
} __attribute__((packed));

</source>

And finally some helper static methods for MMIO read/write operations and Ports I/O

<source lang="c">

class MMIOUtils
{
    public:
        static uint8_t read8 (uint64_t p_address);
        static uint16_t read16 (uint64_t p_address);
        static uint32_t read32 (uint64_t p_address);
        static uint64_t read64 (uint64_t p_address);
        static void write8 (uint64_t p_address,uint8_t p_value);
        static void write16 (uint64_t p_address,uint16_t p_value);
        static void write32 (uint64_t p_address,uint32_t p_value);
        static void write64 (uint64_t p_address,uint64_t p_value);
};


uint8_t MMIOUtils::read8 (uint64_t p_address)
{
    return *((volatile uint8_t*)(p_address));
}
uint16_t MMIOUtils::read16 (uint64_t p_address)
{
    return *((volatile uint16_t*)(p_address));
    
}
uint32_t MMIOUtils::read32 (uint64_t p_address)
{
    return *((volatile uint32_t*)(p_address));
    
}
uint64_t MMIOUtils::read64 (uint64_t p_address)
{
    return *((volatile uint64_t*)(p_address));    
}
void MMIOUtils::write8 (uint64_t p_address,uint8_t p_value)
{
    (*((volatile uint8_t*)(p_address)))=(p_value);
}
void MMIOUtils::write16 (uint64_t p_address,uint16_t p_value)
{
    (*((volatile uint16_t*)(p_address)))=(p_value);    
}
void MMIOUtils::write32 (uint64_t p_address,uint32_t p_value)
{
    (*((volatile uint32_t*)(p_address)))=(p_value);
    
}
void MMIOUtils::write64 (uint64_t p_address,uint64_t p_value)
{
    (*((volatile uint64_t*)(p_address)))=(p_value);    
}

</source>

<source lang="c">
#ifndef PORTS_H_
#define PORTS_H_


class Ports
{
    private:
    public:
        static void outportb (uint16_t p_port,uint8_t data);
        static void outportw (uint16_t p_port,uint16_t data);
        static void outportl (uint16_t p_port,uint32_t data);
        static uint8_t inportb( uint16_t p_port);
        static uint16_t inportw( uint16_t p_port);
        static uint32_t inportl( uint16_t p_port);
};

#endif /* PORTS_H_ */


/* void Ports::outportb (uint16_t p_port,uint8_t p_data)
 * 
 * This method outputs a byte to a hardware port.
 * It uses an inline asm with the volatile keyword
 * to disable compiler optimization.
 * 
 *  p_port: the port number to output the byte p_data to.
 *  p_data: the byte to to output to the port p_port.
 * 
 * Notice the input constraint
 *      "dN" (port) : indicates using the DX register to store the 
 *                  value of port in it
 *      "a"  (data) : store the value of data into 
 * 
 * The above constraint will instruct the compiler to generate assembly
 * code that looks like that
 *      mov    %edi,%edx
 *      mov    %esi,%eax
 *      out    %eax,(%dx)
 * 
 * According the ABI, the edi will have the value of p_port and esi will have
 * the value of the p_data
 * 
 */
void Ports::outportb (uint16_t p_port,uint8_t p_data)
{
    asm volatile ("outb %1, %0" : : "dN" (p_port), "a" (p_data));
}

/* void Ports::outportw (uint16_t p_port,uint16_t p_data)
 * 
 * This method outputs a word to a hardware port.
 * 
 *  p_port: the port number to output the byte p_data to.
 *  p_data: the byte to to output to the port p_port.
 * 
 */


void Ports::outportw (uint16_t p_port,uint16_t p_data)
{
    asm volatile ("outw %1, %0" : : "dN" (p_port), "a" (p_data));
}

/* void Ports::outportl (uint16_t p_port,uint32_t p_data)
 * 
 * This method outputs a double word to a hardware port.
 * 
 *  p_port: the port number to output the byte p_data to.
 *  p_data: the byte to to output to the port p_port.
 * 
 */


void Ports::outportl (uint16_t p_port,uint32_t p_data)
{
    asm volatile ("outl %1, %0" : : "dN" (p_port), "a" (p_data));
}

/* uint8_t Ports::inportb( uint16_t p_port)
 * 
 * This method reads a byte from a hardware port.
 * 
 *  p_port: the port number to read the byte from.
 *  return value : a byte read from the port p_port.
 * 
 * Notice the output constraint "=a", this tells the compiler 
 * to expect the save the value of register AX into the variable l_ret
 * The register AX should contain the result of the inb instruction.
 * 
 * 
 */

uint8_t Ports::inportb( uint16_t p_port)
{
    uint8_t l_ret;
    asm volatile("inb %1, %0" : "=a" (l_ret) : "dN" (p_port));
    return l_ret;
}

/* uint16_t Ports::inportw( uint16_t p_port)
 * 
 * This method reads a word from a hardware port.
 * 
 *  p_port: the port number to read the word from.
 *  return value : a word read from the port p_port.
 * 
 */


uint16_t Ports::inportw( uint16_t p_port)
{
    uint16_t l_ret;
    asm volatile ("inw %1, %0" : "=a" (l_ret) : "dN" (p_port));
    return l_ret;
}


/* uint16_t Ports::inportl( uint16_t p_port)
 * 
 * This method reads a double word from a hardware port.
 * 
 *  p_port: the port number to read the double word from.
 *  return value : a double word read from the port p_port.
 * 
 */

uint32_t Ports::inportl( uint16_t p_port)
{
    uint32_t l_ret;
    asm volatile ("inl %1, %0" : "=a" (l_ret) : "dN" (p_port));
    return l_ret;
}


</source>

== The Driver Class Header (Class Definition)==


<source lang="c">
class E1000 : public NetworkDriver
{
    private:
        
        uint8_t bar_type;     // Type of BAR0
        uint16_t io_base;     // IO Base Address
        uint64_t  mem_base;   // MMIO Base Address
        bool eerprom_exists;  // A flag indicating if eeprom exists
        uint8_t mac [6];      // A buffer for storing the mack address
        struct e1000_rx_desc *rx_descs[E1000_NUM_RX_DESC]; // Receive Descriptor Buffers
        struct e1000_tx_desc *tx_descs[E1000_NUM_TX_DESC]; // Transmit Descriptor Buffers
        uint16_t rx_cur;      // Current Receive Descriptor Buffer
        uint16_t tx_cur;      // Current Transmit Descriptor Buffer
        
        
        // Send Commands and read results From NICs either using MMIO or IO Ports
        void writeCommand( uint16_t p_address, uint32_t p_value);
        uint32_t readCommand(uint16_t p_address);


        bool detectEEProm(); // Return true if EEProm exist, else it returns false and set the eerprom_existsdata member
        uint32_t eepromRead( uint8_t addr); // Read 4 bytes from a specific EEProm Address
        bool readMACAddress();       // Read MAC Address
        void startLink ();           // Start up the network
        void rxinit();               // Initialize receive descriptors an buffers
        void txinit();               // Initialize transmit descriptors an buffers
        void enableInterrupt();      // Enable Interrupts
        void handleReceive();        // Handle a packet reception.
    public:

        E1000(PCIConfigHeader * _pciConfigHeader); // Constructor. takes as a parameter a pointer to an object that encapsulate all he PCI configuration data of the device
        void start ();                             // perform initialization tasks and starts the driver
        void fire (InterruptContext * p_interruptContext);  // This method should be called by the interrupt handler 
        uint8_t * getMacAddress ();                         // Returns the MAC address
        int sendPacket(const void * p_data, uint16_t p_len);  // Send a packet
        ~E1000();                                             // Default Destructor
};
</source>

== How the Gears Move (Class methods implementation) ==

First of all we need to be able to send commands and read results from the NIC. It is important to detect the type of BAR0 and based on that the correct communication mechanism should be adopted. The following two methods encapsulate the read/write commands and uses MMIO or IO ports based on the value in BAR0 which is reflected in bar_type data member flag.

<source lang="c">
void E1000::writeCommand( uint16_t p_address, uint32_t p_value)
{
    if ( bar_type == 0 )
    {
        MMIOUtils::write32(mem_base+p_address,p_value);
    }
    else
    {
        Ports::outportl(io_base, p_address);
        Ports::outportl(io_base + 4, p_value);
    }
}
uint32_t E1000::readCommand( uint16_t p_address)
{
    if ( bar_type == 0 )
    {
        return MMIOUtils::read32(mem_base+p_address);
    }
    else
    {
        Ports::outportl(io_base, p_address);
        return Ports::inportl(io_base + 4);
    }
} 
</source>


Now we need to detect if the card has an EEPROM or not. The Qemu and Bochs emulate EEPROM, but the I217 and 82577LM do not. The following first method tries to read the status field of the EEPROM, the status field should contain the value 0x10, and based on the result the internal data member eerprom_exists. The second method performs a 2-bytes read operation from the EEPROM

<source lang="c">
bool E1000::detectEEProm()
{
    uint32_t val = 0;
    writeCommand(REG_EEPROM, 0x1); 

    for(int i = 0; i < 1000 && ! eerprom_exists; i++)
    {
            val = readCommand( REG_EEPROM);
            if(val & 0x10)
                    eerprom_exists = true;
            else
                    eerprom_exists = false;
    }
    return eerprom_exists;
}

uint32_t E1000::eepromRead( uint8_t addr)
{
	uint16_t data = 0;
	uint32_t tmp = 0;
        if ( eerprom_exists)
        {
            	writeCommand( REG_EEPROM, (1) | ((uint32_t)(addr) << 8) );
        	while( !((tmp = readCommand(REG_EEPROM)) & (1 << 4)) );
        }
        else
        {
            writeCommand( REG_EEPROM, (1) | ((uint32_t)(addr) << 2) );
            while( !((tmp = readCommand(REG_EEPROM)) & (1 << 1)) );
        }
	data = (uint16_t)((tmp >> 16) & 0xFFFF);
	return data;
}

</source>



The first thing you will need to do after detecting the BAR0 type and the existence of the EEPROM is to read the hardware MAC address of the NIC. The following method reads the hardware mac address based. If an EEPROM exists it will read it from the EEPROM else it will read it from address 0x5400 where it should be located in that case. It is very important to detect if an EEPROM exists or not prior to reading the MAC address.

<source lang="c">

bool E1000::readMACAddress()
{
    if ( eerprom_exists)
    {
        uint32_t temp;
        temp = eepromRead( 0);
        mac[0] = temp &0xff;
        mac[1] = temp >> 8;
        temp = eepromRead( 1);
        mac[2] = temp &0xff;
        mac[3] = temp >> 8;
        temp = eepromRead( 2);
        mac[4] = temp &0xff;
        mac[5] = temp >> 8;
    }
    else
    {
        uint8_t * mem_base_mac_8 = (uint8_t *) (mem_base+0x5400);
        uint32_t * mem_base_mac_32 = (uint32_t *) (mem_base+0x5400);
        if ( mem_base_mac_32[0] != 0 )
        {
            for(int i = 0; i < 6; i++)
            {
                mac[i] = mem_base_mac_8[i];
            }
        }
        else return false;
    }
    return true;
}
</source>


Now, we need to configure the transmit and receive descriptor buffers, here are the implementation of the corresponding methods. The rxinit method is identical to the one I use for my e1000 driver. The difference is in txinit

<source lang="c">

void E1000::rxinit()
{
    uint8_t * ptr;
    struct e1000_rx_desc *descs;

    // Allocate buffer for receive descriptors. For simplicity, in my case khmalloc returns a virtual address that is identical to it physical mapped address.
    // In your case you should handle virtual and physical addresses as the addresses passed to the NIC should be physical ones
 
    ptr = (uint8_t *)(kmalloc_ptr->khmalloc(sizeof(struct e1000_rx_desc)*E1000_NUM_RX_DESC + 16));

    descs = (struct e1000_rx_desc *)ptr;
    for(int i = 0; i < E1000_NUM_RX_DESC; i++)
    {
        rx_descs[i] = (struct e1000_rx_desc *)((uint8_t *)descs + i*16);
        rx_descs[i]->addr = (uint64_t)(uint8_t *)(kmalloc_ptr->khmalloc(8192 + 16));
        rx_descs[i]->status = 0;
    }

    writeCommand(REG_TXDESCLO, (uint32_t)((uint64_t)ptr >> 32) );
    writeCommand(REG_TXDESCHI, (uint32_t)((uint64_t)ptr & 0xFFFFFFFF));

    writeCommand(REG_RXDESCLO, (uint64_t)ptr);
    writeCommand(REG_RXDESCHI, 0);

    writeCommand(REG_RXDESCLEN, E1000_NUM_RX_DESC * 16);

    writeCommand(REG_RXDESCHEAD, 0);
    writeCommand(REG_RXDESCTAIL, E1000_NUM_RX_DESC-1);
    rx_cur = 0;
    writeCommand(REG_RCTRL, RCTL_EN| RCTL_SBP| RCTL_UPE | RCTL_MPE | RCTL_LBM_NONE | RTCL_RDMTS_HALF | RCTL_BAM | RCTL_SECRC  | RCTL_BSIZE_8192);
    
}


void E1000::txinit()
{    
    uint8_t *  ptr;
    struct e1000_tx_desc *descs;
    // Allocate buffer for receive descriptors. For simplicity, in my case khmalloc returns a virtual address that is identical to it physical mapped address.
    // In your case you should handle virtual and physical addresses as the addresses passed to the NIC should be physical ones
    ptr = (uint8_t *)(kmalloc_ptr->khmalloc(sizeof(struct e1000_tx_desc)*E1000_NUM_TX_DESC + 16));

    descs = (struct e1000_tx_desc *)ptr;
    for(int i = 0; i < E1000_NUM_TX_DESC; i++)
    {
        tx_descs[i] = (struct e1000_tx_desc *)((uint8_t*)descs + i*16);
        tx_descs[i]->addr = 0;
        tx_descs[i]->cmd = 0;
        tx_descs[i]->status = TSTA_DD;
    }

    writeCommand(REG_TXDESCHI, (uint32_t)((uint64_t)ptr >> 32) );
    writeCommand(REG_TXDESCLO, (uint32_t)((uint64_t)ptr & 0xFFFFFFFF));


    //now setup total length of descriptors
    writeCommand(REG_TXDESCLEN, E1000_NUM_TX_DESC * 16);


    //setup numbers
    writeCommand( REG_TXDESCHEAD, 0);
    writeCommand( REG_TXDESCTAIL, 0);
    tx_cur = 0;
    writeCommand(REG_TCTRL,  TCTL_EN
        | TCTL_PSP
        | (15 << TCTL_CT_SHIFT)
        | (64 << TCTL_COLD_SHIFT)
        | TCTL_RTLC);

    // This line of code overrides the one before it but I left both to highlight that the previous one works with e1000 cards, but for the e1000e cards 
    // you should set the TCTRL register as follows. For detailed description of each bit, please refer to the Intel Manual.
    // In the case of I217 and 82577LM packets will not be sent if the TCTRL is not configured using the following bits.
    writeCommand(REG_TCTRL,  0b0110000000000111111000011111010);
    writeCommand(REG_TIPG,  0x0060200A);

}


</source>


To enable interrupts 
<source lang="c">
void E1000::enableInterrupt()
{
    writeCommand(REG_IMASK ,0x1F6DC);
    writeCommand(REG_IMASK ,0xff & ~4);
    readCommand(0xc0);

}
</source>

As we have defined most of the building blocks and the helper methods lets define the main methods of the class.


The constructor is responsible for fetching PCI related data and initialize the object internal state

<source lang="c">
E1000::E1000(PCIConfigHeader * p_pciConfigHeader) : NetworkDriver(p_pciConfigHeader)
{
    // Get BAR0 type, io_base address and MMIO base address
    bar_type = pciConfigHeader->getPCIBarType(0);
    io_base = pciConfigHeader->getPCIBar(PCI_BAR_IO) & ~1;
    mem_base = pciConfigHeader->getPCIBar( PCI_BAR_MEM) & ~3;    
   
    // Off course you will need here to map the memory address into you page tables and use corresponding virtual addresses

    // Enable bus mastering
    pciConfigHeader->enablePCIBusMastering();
    eerprom_exists = false;
}
</source>

The start method basically detects the EEPROM, reads the MAC addresses, setup rx and tx buffers, register the interrupt handler, and enable NIC interrupts


<source lang="c">

bool E1000::start ()
{
    detectEEProm ();
    if (! readMACAddress()) return false;
    printMac();
    startLink();
    
    for(int i = 0; i < 0x80; i++)
        writeCommand(0x5200 + i*4, 0);
    if ( interruptManager->registerInterrupt(IRQ0+pciConfigHeader->getIntLine(),this))
    {
        enableInterrupt();
        rxinit();
        txinit();        
        video.putString("E1000 card started\n",COLOR_RED,COLOR_WHITE);
        return true;
    }
    else return false;
}


</source>


Your interrupt handler should eventually call the fire method which handles the NIC's events
<source lang="c">

void E1000::fire (InterruptContext * p_interruptContext)
{
    if ( p_interruptContext->getInteruptNumber() == pciConfigHeader->getIntLine()+IRQ0)
    {        
        /* This might be needed here if your handler doesn't clear interrupts from each device and must be done before EOI if using the PIC.
           Without this, the card will spam interrupts as the int-line will stay high. */
        writeCommand(REG_IMASK, 0x1);
       
        uint32_t status = readCommand(0xc0);
        if(status & 0x04)
        {
            startLink();
        }
        else if(status & 0x10)
        {
           // good threshold
        }
        else if(status & 0x80)
        {
            handleReceive();
        }
    }
}

void E1000::handleReceive()
{
    uint16_t old_cur;
    bool got_packet = false;
 
    while((rx_descs[rx_cur]->status & 0x1))
    {
            got_packet = true;
            uint8_t *buf = (uint8_t *)rx_descs[rx_cur]->addr;
            uint16_t len = rx_descs[rx_cur]->length;

            // Here you should inject the received packet into your network stack


            rx_descs[rx_cur]->status = 0;
            old_cur = rx_cur;
            rx_cur = (rx_cur + 1) % E1000_NUM_RX_DESC;
            writeCommand(REG_RXDESCTAIL, old_cur );
    }    
}

</source>

Finally we define the sendPacket method as follows

<source lang="c">

int E1000::sendPacket(const void * p_data, uint16_t p_len)
{    
    tx_descs[tx_cur]->addr = (uint64_t)p_data;
    tx_descs[tx_cur]->length = p_len;
    tx_descs[tx_cur]->cmd = CMD_EOP | CMD_IFCS | CMD_RS;
    tx_descs[tx_cur]->status = 0;
    uint8_t old_cur = tx_cur;
    tx_cur = (tx_cur + 1) % E1000_NUM_TX_DESC;
    writeCommand(REG_TXDESCTAIL, tx_cur);   
    while(!(tx_descs[old_cur]->status & 0xff));    
    return 0;
}

</source>


This is an example of how to instantiate an object of this class and startup you driver. I assume that you have scanned your PCI buses and loaded the found devices parameters into some data structures; in our example this is done by the PCIConfigManager class, which is outside the scope of this tutorial

<source lang="c">

        pciConfigHeaderManager.initialize(); // Initialize the PCIConfigHeaderManager Object and scan PCI devices
        if ( e1000PCIConfigHeader == NULL ) e1000PCIConfigHeader = pciConfigHeaderManager.getPCIDevice(INTEL_VEND,E1000_I217);
        if ( e1000PCIConfigHeader == NULL ) e1000PCIConfigHeader = pciConfigHeaderManager.getPCIDevice(INTEL_VEND,E1000_82577LM);
        if ( e1000PCIConfigHeader != NULL )
        {
            E1000 * e1000 = new E1000(e1000PCIConfigHeader);
            if (!e1000->start())
            {
               // Error starting the NIC
            }
        }
        else
        {
           // Intel cards not found
        }

</source>

== Summary and Wrap Up ==

I have presented in this Wiki the steps I followed to make an e1000 driver work with the two e1000e NICs Intel I217 and 82577LM. The wiki does not show how to utilize all the features of the NICs, but basically primitive setup and send/receive packets. Three important issues that I faced:

* You need to detect the BAR0 type as some cards uses IO ports and others uses MMIO and you need to communincate with the NIC with the method appropriate to each.
* You need to check if the card supports an EEPROM and read the MAC address from the EEPROM if the card supports it or read it from address 0x5400 if it does not support an EEPROM
* You need to make sure to setup the card TCTRL register (Transmission Control Register) with the value 0b0110000000000111111000011111010. For more details reference the Intel manual for the meaning of the different bits of the register

== Manuals ==
These are the full Intel manuals:
[http://www.intel.com/content/www/us/en/ethernet-controllers/ethernet-controller-i217-spec-update.html Intel Ethernet i217 V]
[http://www.intel.com/content/www/us/en/ethernet-controllers/82577-gbe-phy-datasheet.html Intel 82577 Gigabit Ethernet PHY]

(Looks like the former manual was moved [http://www.intel.com/content/www/us/en/embedded/products/networking/i217-ethernet-controller-datasheet.html here]).

[[Category:Network Hardware]]
````

<a id="w003769"></a>

## W003769 — User talk:Kemosparc

Bron-ID: W003769
Namespace: User talk (3)
Revisie: 17900 | Datum: 2015-04-18T23:26:48Z
Bron: https://wiki.osdev.org/User_talk:Kemosparc
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AKemosparc&oldid=17900
Doorverwijzing: Intel Ethernet i217

````text
#REDIRECT [[Intel Ethernet i217]]
````

<a id="w003770"></a>

## W003770 — User:Sortie/Yes Another Unix Clone

Bron-ID: W003770
Namespace: User (2)
Revisie: 19029 | Datum: 2016-03-30T12:00:32Z
Bron: https://wiki.osdev.org/User:Sortie/Yes_Another_Unix_Clone
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ASortie%2FYes_Another_Unix_Clone&oldid=19029

````text
I occasionally encounter the statement

:''Really, another Unix clone?''

in the replies to announcements of hobbyist operating systems. This always rubbed me the wrong way, and I'd like to examine why.

== You are being rude ==

Developers that went through the effort to make and publish their work deserve recognition. They did the work for their own reasons. If you post a comment like that, you implicitly carry the connotation that their effort is wasted and rudely dismiss it. Surely you don't comment on other projects "Meh, should have used Lisp instead of PHP". Do you submit such remarks to the Linux kernel mailing list? Hobbyist operating systems development is seriously under-appreciated whenever someone publishes their work.

== Offtopic ==

You are also entirely offtopic. You are advancing your own agenda and the only thing your comment has to do with the operating system is the useless fact that it has at least some Unix compatibility. Have you even tried it out? Read the code? Cared to comment on anything unique?

== Personal Utility Function ==

Hobbyist operating systems developers do the work for their own personal reasons. If they want to clone MS-DOS, that's up to them. Likewise if they want to make an xkcd 528 compliant operating system or even be POSIX compatible. Some people are in it for the fun, or perhaps to learn, or even challenge themselves. You don't get to tell them what to do. Try instead help them achieve what they really want rather than being helpfully unhelpful.

== Innovation ==

There's also the common variant

:''Another Unix clone, where is the innovation?''

as if being compatible implies an inability to innovate. That's definitely not the case. If you provide a C interface, modern POSIX is a very reasonable and minimal system interface. The odd awful parts are generally optionally for this reason, and it's even possible to be compatible with the subset of POSIX that you happen to like. The system interface is actually a rather limited part of the operating system, the rest of the system is generally unconstrained.

You can still be a microkernel or a distributed networked system. You can add address layout randomization and remote procedure calls. You can still add a new interpreted/compiled language or multiple operating system personalities. If you want to be of higher quality and security than Linux that's entirely feasible.

But who actually needs innovation at the operating system interface level? By design it's boring primitives that can be used to do interesting stuff. If you create your own system interface, your result will likely be equivalent but different. A custom design should at least be an order magnitude better than POSIX to be worthwhile.

But perhaps there actually is an actual issue of lacking innovation. Somehow, there are things that must be innovated at the operating system level, which all the application level software has failed to do. It turns out to be a major problem for society. Why are you discussing this in a topic about a hobbyist operating system? Go gather evidence, find out what things needs to be innovated, and make your own serious debate separately.

== Custom Designs ==

Let us assume that the developer takes your suggetion and attempts to make a wholly new operating system design. Surely the POSIX API has to go. A lot of concepts probably needs to be rethought. The C programming language is probably unsuitable, why not create a new programming language that better fits the new operating system? Then we need to create a new compiler or interpreter. We need to write a lot of core software from scratch. Design mistakes have to be identified and revised. When you are finally done, you are an expert in a custom operating system, and all that knowledge is useless unless you managed to gain market share.

But that's a wholly different project than the developer set out to do and it is a couple of magnitudes more work. This kind of operating systems work is impressive, no doubt, but it's also some of the very hardest. The developer has a different utility function than you.

== Advantages ==

Implementing POSIX has a lot of advantages for hobbyist developers. It gives them a concrete interface they have to implement, a framework to work within. That's useful and lets them know whether they have reached some level of completeness.

Secondly, API compatibility allows porting a lot of third party software. Porting a libc or compiler is a serious short cut if you want to achieve self-hosting status in a modest timeframe. It's really an accomplishment to do this even if straightforward in principle. Implementing a known API makes the system more familiar to users and potential contributors. You get a lot of system documentation for free. If you build a POSIX system, that shows you really do know POSIX. That's a big plus and knowledge that's actually useful in the real world.
So: Yes. Another Unix Clone.
````

<a id="w003771"></a>

## W003771 — User:Max/Ghost Kernel

Bron-ID: W003771
Namespace: User (2)
Revisie: 27463 | Datum: 2022-10-10T15:18:51Z
Bron: https://wiki.osdev.org/User:Max/Ghost_Kernel
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AMax%2FGhost_Kernel&oldid=27463

````text
{{Infobox Project
|name=Ghost OS

|image=ghost-ui-preview.png
|caption=User interface showing a few windows & components

|developers=[[User:max|Max Schlüssel]]

|kerneltype=micro-kernel
|platform=x86
|taskmodel=Preemptive multitasking
|language=C++, assembly

|website=[https://ghostkernel.org ghostkernel.org]
}}

'''Ghost OS''' is a free and open-source hobby operating systemIt is under development since 2014 and currently compatible with the x86 platform.

The system is based on a micro kernel named "Ghost Kernel" and features symmetric multi-processing and multitasking. Most of the kernel and system programs are written in C/C++. The project contains various advanced features like a graphical user interface and a custom C library.

The sources of the project can be found on [https://github.com/maxdev1/ghost GitHub].
````

<a id="w003772"></a>

## W003772 — File:Ghost-ui-preview.png

Bron-ID: W003772
Namespace: File (6)
Revisie: 17937 | Datum: 2015-04-28T13:44:21Z
Bron: https://wiki.osdev.org/File:Ghost-ui-preview.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AGhost-ui-preview.png&oldid=17937

````text

````

<a id="w003773"></a>

## W003773 — Endian

Bron-ID: W003773
Namespace: Main (0)
Revisie: 17959 | Datum: 2015-04-29T19:00:54Z
Bron: https://wiki.osdev.org/Endian
Revisie-URL: https://wiki.osdev.org/index.php?title=Endian&oldid=17959
Doorverwijzing: Endianness

````text
#REDIRECT [[Endianness]]
````
