package com.memohai.autofish.services.system

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class AppControllerImplTest {

    @Test
    fun `parseTopActivityOutput strips trailing brace artifacts`() {
        val line = "topResumedActivity=ActivityRecord{af6f2a4 u0 org.mozilla.firefox/.App t83}"
        val parsed = AppControllerImpl.parseTopActivityOutput(line)
        assertEquals("org.mozilla.firefox/.App", parsed)
    }

    @Test
    fun `parseTopActivityOutput returns null for blank`() {
        assertNull(AppControllerImpl.parseTopActivityOutput("   "))
    }
}

