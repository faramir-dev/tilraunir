Ubuntu 20.04 Build
==================

              # apt update
              # apt install build-essential git cmake libglib2.0-dev libgcrypt-dev libc-ares-dev libpcap-dev bison flex  qttools5-dev qtmultimedia5-dev
              $ cd /path/to/tezos/dissector/t3z0s/t3z0s_rs &&
                cargo clean && cargo build &&
                cp -v target/debug/libt3z0s_rs.a /usr/lib/
                && cd -
              $ git clone https://github.com/wireshark/wireshark.git
              $ cd wireshark/plugins/epan &&
                ln -s /path/to/tezos/dissector/t3z0s . &&
                cd -
              $ mkdir build && cd build
        build $ cmake ../wireshark
        build $ make
              # mkdir /usr/local/lib/wireshark/plugins/3.3 &&
                cd /usr/local/lib/wireshark/plugins/3.3 &&
                ln -s /path/to/wireshark/build/run/plugins/3.3/epan . &&
                cd -
        build $ run/tshark

Verify that plugin is installed
-------------------------------

               $ run/tshark -G plugins | grep -i t3z0s
               t3z0s.so 0.0.1 dissector /usr/local/lib/wireshark/plugins/3.3/epan/t3z0s.so
               $ run/tshark -G protocols | grep -i t3z0s
               T3z0s Protocol  t3z0s   t3z0s

Simple Session
--------------

- Terminal 1:

               $ run/tshark -i lo -w /tmp/xyz.pcap


- Terminal 2:

               $ nc -lvu 1024

- Terminal 3:

               $ echo 'Hello World YXZ' | nc -u 127.0.0.1 1024

- Continue in Terminal 1:

               $ run/tshark -r /tmp/xyz.pcap
                   1 0.000000000    127.0.0.1 ? 127.0.0.1    t3z0s 58
                   ...
                   3 1.147193109    127.0.0.1 ? 127.0.0.1    t3z0s 58
                   ...
                   5 1.842787799    127.0.0.1 ? 127.0.0.1    t3z0s 58

                $ run/tshark -Vr /tmp/xyz.pcap
                   ...
                Internet Control Message Protocol
                    Type: 3 (Destination unreachable)
                    Code: 3 (Port unreachable)
                    Checksum: 0x4951 [correct]
                    [Checksum Status: Good]
                    Unused: 00000000
                    Internet Protocol Version 4, Src: 127.0.0.1, Dst: 127.0.0.1
                        0100 .... = Version: 4
                        .... 0101 = Header Length: 20 bytes (5)
                        Differentiated Services Field: 0x00 (DSCP: CS0, ECN: Not-ECT)
                            0000 00.. = Differentiated Services Codepoint: Default (0)
                            .... ..00 = Explicit Congestion Notification: Not ECN-Capable Transport (0)
                        Total Length: 44
                        Identification: 0xd646 (54854)
                        Flags: 0x40, Don't fragment
                            0... .... = Reserved bit: Not set
                            .1.. .... = Don't fragment: Set
                            ..0. .... = More fragments: Not set
                        Fragment offset: 0
                        Time to live: 64
                        Protocol: UDP (17)
                        Header checksum: 0x6678 [validation disabled]
                        [Header checksum status: Unverified]
                        Source: 127.0.0.1
                        Destination: 127.0.0.1
                    User Datagram Protocol, Src Port: 44054, Dst Port: 1024
                        Source Port: 44054
                        Destination Port: 1024
                        Length: 24
                        Checksum: 0xfe2b [unverified]
                        [Checksum Status: Unverified]
                        [Stream index: 2]
                        UDP payload (16 bytes)
                T3z0s Protocol
                    T3z0s Phrase: Hello World YXZ\n
                    T3z0s Word: Hello
                    T3z0s Word: World
                    T3z0s Word: YXZ