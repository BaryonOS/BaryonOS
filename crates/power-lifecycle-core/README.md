# Power lifecycle core

This standalone `no_std` state machine models generic power transitions:
active/idle, suspend/cancel/failure, resume/failure, reset, and failed
(recoverable only via reset). Transition requests carry a generation so a
request prepared for an older state cannot be applied after another transition.

It has no ACPI, device callbacks, scheduling, flush protocol, or native
consumer. Host tests establish only transition-model behavior, not hardware
sleep/wake or power-failure safety.
