import QtQuick 2.4

Item {
    width: 800
    height: 800

    Grid {
        id: grid
        columns: 5
        anchors.fill: parent

        Item {
            id: _item0
            width: 300
            height: 100
        }

        Text {
            id: element
            text: qsTr("Name:")
            font.pixelSize: 12
        }

        Rectangle {
            id: _item1
            width: 200
            height: 1
        }

        TextInput {
            id: textEdit
            width: 80
            height: 20
            text: qsTr("EvythingWorks
")
            font.pixelSize: 12
        }

        Item {
            id: _item3
            width: 200
            height: 1
        }

        Item {
            id: element1
            width: 200
            height: 100
        }

        Text {
            id: element2
            text: qsTr("Email")
            font.pixelSize: 12
        }

        Item {
            id: element3
            width: 200
            height: 100
        }

        TextInput {
            id: textInput
            width: 80
            height: 20
            text: qsTr("Text Input")
            font.pixelSize: 12
        }

        Item {
            id: element4
            width: 200
            height: 100
        }
    }
}

/*##^##
Designer {
    D{i:1;anchors_height:400;anchors_width:400;anchors_x:147;anchors_y:164}
}
##^##*/

