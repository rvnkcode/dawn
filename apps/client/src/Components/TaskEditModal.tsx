import { UpdateTaskDto } from "../../Api";
import { api, listAtom, isModalOpenAtom, modalInputValueAtom, selectedTaskIdAtom } from "../selectors/task";
import { EditFilled } from "@ant-design/icons";
import { Button, Form, Input, message, Modal } from "antd";
import { useRef, useState } from "react";
import Draggable from "react-draggable";
import type { DraggableData, DraggableEvent } from "react-draggable";
import { useRecoilState, useRecoilValue } from "recoil";

export default function TaskEditModal() {
  // Draggable
  const draggableRef = useRef<HTMLDivElement>(null);
  const [bounds, setBounds] = useState({ left: 0, top: 0, bottom: 0, right: 0 });
  const [isDragEnabled, setIsDragEnabled] = useState(false);

  const onModalStart = (_event: DraggableEvent, uiData: DraggableData) => {
    const { clientWidth, clientHeight } = window.document.documentElement;
    const targetRect = draggableRef.current?.getBoundingClientRect();
    if (!targetRect) {
      return;
    }

    setBounds({
      left: -targetRect.left + uiData.x,
      right: clientWidth - (targetRect.right - uiData.x),
      top: -targetRect.top + uiData.y,
      bottom: clientHeight - (targetRect.bottom - uiData.y)
    });
  };

  const [list, setList] = useRecoilState(listAtom);
  const selectedTaskId = useRecoilValue(selectedTaskIdAtom);
  const [isModalOpen, setIsModalOpen] = useRecoilState(isModalOpenAtom);
  const modalInputValue = useRecoilValue(modalInputValueAtom);

  const editTask = (value: UpdateTaskDto) => {
    api.task
      .appControllerUpdateTask(selectedTaskId, value)
      .then((response) => {
        const updateList = [...list];
        updateList[updateList.findIndex((task) => task.id === +selectedTaskId)] = response.data;
        setList(updateList);
        setIsModalOpen(false);
      })
      .catch((error) => {
        console.error(error);
        message.error(`Cannot update task`);
      });
  };

  return (
    <Modal
      title={
        <div
          onMouseOver={() => {
            if (!isDragEnabled) {
              setIsDragEnabled(true);
            }
          }}
          onMouseOut={() => {
            setIsDragEnabled(false);
          }}
        >
          Edit To-Do
        </div>
      }
      open={isModalOpen}
      footer={null}
      onCancel={() => {
        setIsModalOpen(false);
      }}
      closable={false}
      destroyOnClose={true}
      modalRender={(modal) => (
        <Draggable disabled={isDragEnabled} bounds={bounds} onStart={(event, uiData) => onModalStart(event, uiData)}>
          <div ref={draggableRef}>{modal}</div>
        </Draggable>
      )}
    >
      <Form
        onFinish={editTask}
        style={{
          cursor: "move"
        }}
      >
        <Form.Item name="title" initialValue={modalInputValue}>
          <Input required placeholder="To-Do" autoComplete="off" />
        </Form.Item>
        <div style={{ textAlign: "right" }}>
          <Button htmlType="submit" type="primary">
            <EditFilled />
          </Button>
        </div>
      </Form>
    </Modal>
  );
}
