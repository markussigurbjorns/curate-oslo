let scene, camera, renderer, controls;

function init() {
  // 1. Create the scene
  scene = new THREE.Scene();

  // 2. Create a camera
  camera = new THREE.PerspectiveCamera(
    45, 
    window.innerWidth / window.innerHeight,
    0.1, 
    1000
  );
  // Move camera back a bit so we can see the model
  camera.position.set(0, 1, 5);

  // 3. Create the renderer
  renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(window.innerWidth, window.innerHeight);
  // Append the canvas to our container
  document.getElementById('canvas-container').appendChild(renderer.domElement);

  // 4. Add a simple light
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
  scene.add(ambientLight);
  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.6);
  directionalLight.position.set(5, 10, 7).normalize();
  scene.add(directionalLight);

  // 5. Add OrbitControls for click-and-drag rotation
  controls = new THREE.OrbitControls(camera, renderer.domElement);
  // Optional: set some limits, e.g., how close or far you can zoom:
  // controls.minDistance = 1;
  // controls.maxDistance = 20;

  // 6. Load the OBJ model
  const objLoader = new THREE.OBJLoader();
  objLoader.load(
    'model.obj',        // Path to your OBJ file
    function (object) {
      // Center the model if needed
      // By default, many OBJ files are already centered, 
      // but you might do: object.position.set(0, 0, 0);
      scene.add(object);
    },
    function (xhr) {
      console.log((xhr.loaded / xhr.total) * 100 + '% loaded');
    },
    function (error) {
      console.error('An error happened while loading the .OBJ:', error);
    }
  );

  // 7. Listen for window resizing
  window.addEventListener('resize', onWindowResize);

  // 8. Kick off the render loop
  animate();
}

function onWindowResize() {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
}

// The main render loop
function animate() {
  requestAnimationFrame(animate);

  // Let OrbitControls handle camera movement
  controls.update();

  // Render the scene
  renderer.render(scene, camera);
}

// Initialize everything
init();
