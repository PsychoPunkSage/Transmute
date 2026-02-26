# JNI symbols must not be stripped — the JVM looks them up by name at runtime.
-keep class com.transmute.TransmuteLib { *; }

# Keep native method declarations so R8 doesn't remove them.
-keepclasseswithmembernames class * {
    native <methods>;
}
