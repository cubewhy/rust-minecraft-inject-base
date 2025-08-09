package org.cubewhy;

import java.io.File;

@SuppressWarnings("unused")
public class TweakEntrypoint {
    public static void init(String arg) throws Exception {
        System.out.println("[Tweaker] Welcome to the tweaker");
        System.out.println("[Tweaker] args: " + arg);

        switch (arg) {
            case "dumpclass":
                File baseDir = new File(System.getProperty("user.home"), ".cubewhy/tweaks/dumped-classes");
                new ClassDumper(baseDir).dumpAll();
                break;
        }
    }
}
