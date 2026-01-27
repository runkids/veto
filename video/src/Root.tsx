import { Composition } from "remotion";
import { VetoIntro } from "./compositions/VetoIntro";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="VetoIntro"
        component={VetoIntro}
        durationInFrames={540}
        fps={30}
        width={1920}
        height={1080}
      />
    </>
  );
};
