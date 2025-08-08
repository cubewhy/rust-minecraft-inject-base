package org.cubewhy;

import javax.swing.*;

@SuppressWarnings("unused")
public class TweakEntrypoint {
    public static void init() {
        new Thread(() -> JOptionPane.showMessageDialog(null, "Hello World ")).start();
    }
}
