package com.example;

import java.io.Serializable;

public class Main implements Serializable, Cloneable {

    public static final int ZERO = 0;

    public static final float ZERO_DOT_ZERO = 0.0f;

    public static final long ZERO64 = 0L;

    public static final double ZERO_DOT_ZERO64 = 0.0d;

    public static final String CONDY_PLEASE = "Condy" + "!!!";

    public static void main() throws Exception {
        Runnable indyPlease = () -> System.out.println("Hello, World!");
        indyPlease.run();
    }
}
