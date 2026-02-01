#!/usr/bin/env swift
// veto Touch ID helper
// Compiled binary will show "VetoAuth" in Touch ID dialog

import LocalAuthentication
import Foundation

let reason = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1]
    : "Veto: Approve running this command?"

let context = LAContext()
var error: NSError?

// Try biometrics first
if context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {
    let semaphore = DispatchSemaphore(value: 0)
    var success = false

    context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: reason) { result, _ in
        success = result
        semaphore.signal()
    }

    semaphore.wait()
    print(success ? "AUTH_SUCCESS" : "AUTH_FAILED")
    exit(success ? 0 : 1)
}

// Fallback to device passcode
if context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) {
    let semaphore = DispatchSemaphore(value: 0)
    var success = false

    context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: reason) { result, _ in
        success = result
        semaphore.signal()
    }

    semaphore.wait()
    print(success ? "AUTH_SUCCESS" : "AUTH_FAILED")
    exit(success ? 0 : 1)
}

print("AUTH_UNAVAILABLE")
exit(2)
